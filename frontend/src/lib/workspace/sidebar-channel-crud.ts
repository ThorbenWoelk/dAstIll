import {
  addChannel,
  addVideo,
  deleteChannel,
  listChannelsWhenAvailable,
} from "$lib/api";
import {
  buildOptimisticChannel,
  removeChannelFromCollection,
  removeChannelId,
  replaceOptimisticChannel,
} from "$lib/workspace/channel-actions";
import { putCachedChannels } from "$lib/workspace/workspace-cache";
import type { SidebarStateOptions } from "./sidebar-state.svelte";
import type { Channel } from "$lib/types";
import { resolveNextChannelSelection } from "./route-helpers";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth/required-notice";
import { looksLikeYouTubeVideoInput } from "$lib/utils/youtube-input";
import type { AddSourceSubmission } from "$lib/workspace/component-props";

type SidebarChannelCrudContext = {
  options: SidebarStateOptions;
  getChannels: () => Channel[];
  getChannelOrder: () => string[];
  getSelectedChannelId: () => string | null;
  setChannels: (channels: Channel[]) => void;
  setChannelOrder: (channelOrder: string[]) => void;
  applyLoadedChannelsState: (
    channels: Channel[],
    channelOrder?: string[],
  ) => void;
  applySelectionState: (options: {
    selectedChannelId?: string | null;
    selectedVideoId?: string | null;
  }) => void;
  clearChannelSelectionState: () => void;
  setAddingChannel: (adding: boolean) => void;
  queueChannelDeletion: (channelId: string) => void;
  clearChannelDeletion: () => void;
  syncChannelOrderFromList: () => void;
  replaceOptimisticChannelId: (tempId: string, realId: string) => void;
  selectChannel: (
    channelId: string,
    videoId?: string | null,
    fromUserInteraction?: boolean,
  ) => Promise<void>;
};

function cacheChannels(options: SidebarStateOptions, channels: Channel[]) {
  const writeChannels =
    options.cacheChannels ??
    ((next: Channel[]) => void putCachedChannels(next));
  writeChannels(channels);
}

export function createSidebarChannelCrudOperations(
  context: SidebarChannelCrudContext,
) {
  async function handleAddChannel(
    input: AddSourceSubmission,
  ): Promise<boolean> {
    const submittedInput =
      typeof input === "string" ? input.trim() : input.input.trim();
    if (!submittedInput) return false;

    context.setAddingChannel(true);
    context.options.onError?.(null);

    if (
      typeof input === "string" &&
      looksLikeYouTubeVideoInput(submittedInput)
    ) {
      try {
        const result = await addVideo(submittedInput);
        const refreshedChannels = await listChannelsWhenAvailable({
          retryDelayMs: 500,
        });
        context.applyLoadedChannelsState(
          refreshedChannels,
          context.getChannelOrder(),
        );
        cacheChannels(context.options, refreshedChannels);

        if (context.options.onVideoAdded) {
          await context.options.onVideoAdded(result);
        } else {
          context.applySelectionState({
            selectedChannelId: result.target_channel_id,
          });
          await context.selectChannel(
            result.target_channel_id,
            result.video.id,
            true,
          );
          await context.options.onSelectVideo(result.video.id, {
            forceReload: true,
          });
        }
        return true;
      } catch (error) {
        if (!presentAuthRequiredNoticeIfNeeded(error)) {
          context.options.onError?.((error as Error).message);
        }
        return false;
      } finally {
        context.setAddingChannel(false);
      }
    }

    const previousChannels = [...context.getChannels()];
    const previousSelectedId = context.getSelectedChannelId();

    const { optimisticChannel, tempId, trimmedInput } =
      buildOptimisticChannel(submittedInput);
    context.setChannels([optimisticChannel, ...context.getChannels()]);
    context.setChannelOrder([tempId, ...context.getChannelOrder()]);

    try {
      const channel = await addChannel(
        typeof input === "string"
          ? trimmedInput
          : { input: trimmedInput, openalex_query: input.openalex_query },
      );
      context.setChannels(
        replaceOptimisticChannel(context.getChannels(), tempId, channel),
      );
      context.replaceOptimisticChannelId(tempId, channel.id);

      cacheChannels(context.options, context.getChannels());

      if (context.options.onChannelAdded) {
        await context.options.onChannelAdded(channel);
      } else {
        context.applySelectionState({ selectedChannelId: channel.id });
      }
      return true;
    } catch (error) {
      context.setChannels(previousChannels);
      context.applySelectionState({
        selectedChannelId: previousSelectedId,
      });
      context.syncChannelOrderFromList();
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        context.options.onError?.((error as Error).message);
      }
      return false;
    } finally {
      context.setAddingChannel(false);
    }
  }

  async function handleDeleteChannel(
    channelId: string,
    isOperator: boolean,
    onAccessRequired: () => void,
  ) {
    if (!isOperator) {
      onAccessRequired();
      return;
    }
    context.queueChannelDeletion(channelId);
  }

  async function confirmDeleteChannel(channelId: string, isOperator: boolean) {
    if (!isOperator) return;

    const previousChannels = [...context.getChannels()];
    const nextChannels = removeChannelFromCollection(
      context.getChannels(),
      channelId,
    );
    context.setChannels(nextChannels);
    context.setChannelOrder(
      removeChannelId(context.getChannelOrder(), channelId),
    );

    if (context.getSelectedChannelId() === channelId) {
      const nextChannelId = resolveNextChannelSelection(
        nextChannels,
        channelId,
      );
      if (nextChannelId) {
        await context.selectChannel(nextChannelId);
      } else {
        context.clearChannelSelectionState();
        context.options.onChannelDeselected?.();
      }
    }

    try {
      await deleteChannel(channelId);
      context.options.onChannelDeleted?.(channelId);
    } catch (error) {
      context.setChannels(previousChannels);
      context.syncChannelOrderFromList();
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        context.options.onError?.((error as Error).message);
      }
    } finally {
      context.clearChannelDeletion();
    }
  }

  return {
    handleAddChannel,
    handleDeleteChannel,
    confirmDeleteChannel,
  };
}

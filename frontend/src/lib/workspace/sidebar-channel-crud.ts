import {
  addChannel,
  addVideo,
  deleteChannel,
  listChannelsWhenAvailable,
} from "$lib/api";
import type { ChannelSyncDepthState } from "$lib/channel-view-cache";
import { applySavedChannelOrder } from "$lib/channel-workspace";
import {
  buildOptimisticChannel,
  removeChannelFromCollection,
  removeChannelId,
  replaceOptimisticChannel,
} from "$lib/workspace/channel-actions";
import { putCachedChannels } from "$lib/workspace-cache";
import type { SidebarStateOptions } from "./sidebar-state.svelte";
import type { Channel, Video } from "$lib/types";
import { resolveNextChannelSelection } from "./route-helpers";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { looksLikeYouTubeVideoInput } from "$lib/utils/youtube-input";

type SidebarChannelCrudContext = {
  options: SidebarStateOptions;
  getChannels: () => Channel[];
  getChannelOrder: () => string[];
  getSelectedChannelId: () => string | null;
  setChannels: (channels: Channel[]) => void;
  setChannelOrder: (channelOrder: string[]) => void;
  setSelectedChannelId: (channelId: string | null) => void;
  setVideos: (videos: Video[]) => void;
  setSyncDepth: (syncDepth: ChannelSyncDepthState | null) => void;
  setAddingChannel: (adding: boolean) => void;
  setChannelIdToDelete: (channelId: string | null) => void;
  setShowDeleteConfirmation: (visible: boolean) => void;
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
  async function handleAddChannel(input: string): Promise<boolean> {
    if (!input.trim()) return false;

    context.setAddingChannel(true);
    context.options.onError?.(null);

    const submittedInput = input.trim();

    if (looksLikeYouTubeVideoInput(submittedInput)) {
      try {
        const result = await addVideo(submittedInput);
        const refreshedChannels = applySavedChannelOrder(
          await listChannelsWhenAvailable({
            retryDelayMs: 500,
          }),
          context.getChannelOrder(),
        );
        context.setChannels(refreshedChannels);
        context.syncChannelOrderFromList();
        cacheChannels(context.options, refreshedChannels);

        if (context.options.onVideoAdded) {
          await context.options.onVideoAdded(result);
        } else {
          context.setSelectedChannelId(result.target_channel_id);
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
      buildOptimisticChannel(input);
    context.setChannels([optimisticChannel, ...context.getChannels()]);
    context.setChannelOrder([tempId, ...context.getChannelOrder()]);

    try {
      const channel = await addChannel(trimmedInput);
      context.setChannels(
        replaceOptimisticChannel(context.getChannels(), tempId, channel),
      );
      context.replaceOptimisticChannelId(tempId, channel.id);

      cacheChannels(context.options, context.getChannels());

      if (context.options.onChannelAdded) {
        await context.options.onChannelAdded(channel);
      } else {
        context.setSelectedChannelId(channel.id);
      }
      return true;
    } catch (error) {
      context.setChannels(previousChannels);
      context.setSelectedChannelId(previousSelectedId);
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
    context.setChannelIdToDelete(channelId);
    context.setShowDeleteConfirmation(true);
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
        context.setSelectedChannelId(null);
        context.setVideos([]);
        context.setSyncDepth(null);
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
      context.setChannelIdToDelete(null);
      context.setShowDeleteConfirmation(false);
    }
  }

  return {
    handleAddChannel,
    handleDeleteChannel,
    confirmDeleteChannel,
  };
}

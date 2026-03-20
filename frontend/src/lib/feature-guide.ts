export type TourStep = {
  selector: string;
  title: string;
  body: string;
  placement?: "top" | "bottom" | "left" | "right";
  prepare?: () => void;
  fallbackSelector?: string;
};

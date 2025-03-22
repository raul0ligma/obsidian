import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap text-sm font-medium transition-all [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 font-mono",
  {
    variants: {
      variant: {
        default:
          "bg-black/70 border border-[#ff00ff] text-[#ff00ff] hover:bg-[#ff00ff]/10 hover:text-[#ff00ff] hover:shadow-[0_0_10px_rgba(255,0,255,0.3)]",
        destructive:
          "bg-black/70 border border-[#ff5555] text-[#ff5555] hover:bg-[#ff5555]/10 hover:text-[#ff5555]",
        outline:
          "border border-[#ff00ff]/20 bg-black/40 text-[#ff00ff]/80 hover:bg-[#ff00ff]/5 hover:border-[#ff00ff]/40 hover:text-[#ff00ff]",
        secondary:
          "bg-black/40 border border-[#ff00ff]/20 text-[#ff00ff]/70 hover:bg-[#ff00ff]/5 hover:text-[#ff00ff]",
        ghost: "text-[#ff00ff]/70 hover:text-[#ff00ff] hover:bg-[#ff00ff]/5",
        link: "text-[#ff00ff] underline-offset-4 hover:underline",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 px-3",
        lg: "h-12 px-8",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp
        className={cn(
          buttonVariants({ variant, size, className }),
          "rounded-none"
        )}
        ref={ref}
        {...props}
      />
    );
  }
);
Button.displayName = "Button";

export { Button, buttonVariants };

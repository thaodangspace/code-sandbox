import * as React from 'react';
import * as TabsPrimitive from '@radix-ui/react-tabs';
import { clsx } from 'clsx';

export const Tabs = TabsPrimitive.Root;

export const TabsList = React.forwardRef<
    React.ElementRef<typeof TabsPrimitive.List>,
    React.ComponentPropsWithoutRef<typeof TabsPrimitive.List>
>(({ className, ...props }, ref) => (
    <TabsPrimitive.List
        ref={ref}
        className={clsx('flex w-full justify-around border-t bg-white', className)}
        {...props}
    />
));
TabsList.displayName = TabsPrimitive.List.displayName;

export const TabsTrigger = React.forwardRef<
    React.ElementRef<typeof TabsPrimitive.Trigger>,
    React.ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>
>(({ className, ...props }, ref) => (
    <TabsPrimitive.Trigger
        ref={ref}
        className={clsx(
            'flex-1 py-2 text-sm font-medium data-[state=active]:border-b-2 data-[state=active]:border-blue-500',
            className
        )}
        {...props}
    />
));
TabsTrigger.displayName = TabsPrimitive.Trigger.displayName;

export const TabsContent = React.forwardRef<
    React.ElementRef<typeof TabsPrimitive.Content>,
    React.ComponentPropsWithoutRef<typeof TabsPrimitive.Content>
>(({ className, ...props }, ref) => (
    <TabsPrimitive.Content ref={ref} className={clsx('h-full w-full', className)} {...props} />
));
TabsContent.displayName = TabsPrimitive.Content.displayName;

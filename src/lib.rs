//! Composable, owner-scoped widgets built on Glory's semantic controls.
//!
//! Applications choose their Glory host independently. See the crate README for
//! controlled state, event ordering, theming, and current accessibility scope.
#![doc = include_str!("../README.md")]

mod accordion;
mod actions;
mod affix;
mod alert;
mod anchor;
mod app_shell;
mod appearance;
pub mod autocomplete;
mod avatar;
mod button;
mod checkbox;
mod checkbox_group;
mod chip;
mod collapsible;
pub mod collection;
mod color_swatch;
pub mod combobox;
mod composite;
mod display;
mod document;
mod field;
mod form;
mod form_state;
mod grid_layout;
pub mod hint;
mod image;
mod indicator;
mod input_group;
mod layout;
mod loading_overlay;
pub mod menu;
mod mixed_checkbox;
mod modals;
pub mod multi_select;
pub mod native_select;
mod navigation;
mod navigation_progress;
mod notifications;
mod number_formatter;
mod overlay;
mod sheet;
pub use sheet::{DrawerSnap, DrawerSnaps};
mod password_input;
pub mod pills_input;
mod progress;
mod radio;
mod scroll_area;
mod segmented_control;
pub mod select;
mod separator;
mod skeleton;
mod spinner;
mod splitter;
mod spoiler;
mod state;
mod stepper;
mod switch;
mod tabs;
pub mod tags_input;
mod text_field;
mod text_input;
mod textarea;
pub mod theme;
mod theme_icon;
mod timeline;
mod toggle;
mod toolbar;
mod typography;
mod visually_hidden;

pub use accordion::{Accordion, AccordionItem, accordion, accordion_item};
pub use actions::{
    ActionIcon, Burger, ButtonGroup, CloseButton, action_icon, burger, button_group,
    close_button,
};
pub use affix::*;
pub use alert::{Alert, AlertSeverity, Announcement, MAX_ALERT_TEXT_BYTES, alert};
pub use anchor::{Anchor, AnchorRequest, anchor};
pub use app_shell::*;
pub use autocomplete::{Autocomplete, autocomplete, autocomplete_option};
pub use avatar::{
    Avatar, AvatarGroup, AvatarShape, MAX_AVATAR_GROUP_ITEMS, avatar, avatar_group,
};
pub use button::{Button, ButtonVariant, button};
pub use checkbox::{Checkbox, checkbox};
pub use checkbox_group::{
    CheckboxGroup, CheckboxItem, MAX_CHECKBOX_GROUP_ITEMS, checkbox_group, checkbox_item,
};
pub use chip::{Chip, ChipGroup, ChipItem, ChipMode, chip, chip_group, chip_item};
pub use collapsible::{Collapsible, collapsible};
pub use collection::{
    Collection, CollectionLimits, CollectionMember, MountedItem, RovingFocus, Typeahead,
    TypeaheadConfig,
};
pub use color_swatch::{ColorSwatch, color_swatch};
pub use combobox::MAX_MULTI_SELECT_VALUES;
pub use combobox::{Combobox, ComboboxFilter, MAX_COMBOBOX_SEARCH_BYTES, combobox};
pub use composite::ContentPolicy;
pub use display::{
    Badge, BadgeVariant, Card, EmptyState, Paper, Pill, ShadowSize, badge, card, empty_state,
    paper, pill,
};
pub use document::{
    Blockquote, Code, DataList, DataListItem, Highlight, Item, Kbd, KbdKey, KbdPlatform,
    MAX_DOCUMENT_ITEMS, Mark, TextList, TextListItem, blockquote, code, data_list,
    data_list_item, highlight, item, kbd, mark, text_list, text_list_item,
};
pub use field::{
    Description, Field, FieldControl, FieldError, Fieldset, Label, description, field,
    field_description, field_error, fieldset, label,
};
pub use form::{Form, FormButton, form, reset_button, submit_button};
pub use form_state::*;
pub use grid_layout::{
    AspectRatio, Grid, GridItem, MAX_GRID_ITEMS, SimpleGrid, aspect_ratio, grid, grid_item,
    simple_grid,
};
pub use hint::{FloatingTarget, HoverCard, MAX_HINT_TEXT_BYTES, Tooltip, hover_card, tooltip};
pub use image::{BackgroundImage, Image, MAX_IMAGE_SOURCE_BYTES, background_image, image};
pub use indicator::{Indicator, IndicatorPosition, indicator};
pub use input_group::{
    InputAddon, InputGroup, InputSection, input_addon, input_group, input_section,
};
pub use layout::{
    Box, Center, Container, Flex, Group, Space, Stack, box_, center, container, flex, group,
    space, stack,
};
pub use loading_overlay::{LoadingOverlay, Overlay, loading_overlay, overlay};
pub use menu::{
    DropdownMenu, Menu, MenuEntry, MenuItem, MenuRadioGroup, MenuRadioItem, MenuSubmenu,
    dropdown_menu, menu, menu_checkbox, menu_item, menu_radio_group, menu_radio_item,
    menu_submenu,
};
pub use mixed_checkbox::{CheckState, MixedCheckbox, mixed_checkbox};
pub use modals::{
    MAX_PENDING_MODALS, ModalOutcome, ModalRequest, ModalToken, ModalsHandle, ModalsManager,
    confirmation_request, modal_request, modals_manager,
};
pub use multi_select::{MultiSelect, MultiSelectHandle, multi_select};
pub use native_select::{
    MAX_NATIVE_SELECT_OPTIONS, NativeOption, NativeOptionGroup, NativeSelect, native_select,
    select_group, select_option,
};
pub use navigation::*;
pub use navigation_progress::{
    NavigationProgress, NavigationProgressHandle, NavigationProgressSnapshot,
    NavigationProgressToken, navigation_progress,
};
pub use notifications::{
    MAX_NOTIFICATIONS, MAX_VISIBLE_NOTIFICATIONS, Notification, NotificationClock,
    NotificationCloseReason, NotificationRequest, NotificationToken, NotificationsHandle,
    Toaster, notification, notification_request, notifications, toaster,
};
pub use number_formatter::{
    Currency, NumberFormat, NumberFormatter, NumberLocale, number_formatter,
};
pub use overlay::{
    AlertCancelReason, AlertDialog, Dialog, Drawer, DrawerSide, FloatingDialog, Popover,
    PopoverAlign, alert_dialog, dialog, drawer, floating_dialog, popover,
};
pub use password_input::{PasswordInput, password_input};
pub use pills_input::{PillsInput, pills_input};
pub use progress::{Progress, progress, ring_progress, semi_circle_progress};
pub use radio::{MAX_RADIO_GROUP_ITEMS, Radio, RadioGroup, radio, radio_group};
pub use scroll_area::*;
pub use segmented_control::{Segment, SegmentedControl, segment, segmented_control};
pub use select::{Select, select};
pub use separator::{Separator, separator};
pub use skeleton::{Skeleton, skeleton};
pub use spinner::{Spinner, loader, spinner};
pub use splitter::*;
pub use spoiler::{MAX_SPOILER_BYTES, Spoiler, spoiler};
pub use stepper::*;
pub use switch::{Switch, switch};
pub use tabs::{Tab, TabActivation, Tabs, tab, tabs};
pub use tags_input::{
    MAX_TAGS_INPUT_TAG_BYTES, MAX_TAGS_INPUT_TOTAL_BYTES, MAX_TAGS_INPUT_VALUES,
    TagDuplicatePolicy, TagRejection, TagsInput, TagsInputState, tags_input,
};
pub use text_field::{TextField, text_field};
pub use textarea::{Textarea, textarea};
pub use theme::{ColorScheme, Hue, Size, Theme};
pub use theme_icon::{Icon, ThemeIcon, ThemeIconVariant, theme_icon};
pub use timeline::{
    MAX_TIMELINE_ITEMS, Timeline, TimelineItem, TimelineProgress, timeline, timeline_item,
};
pub use toggle::{
    MAX_TOGGLE_GROUP_ITEMS, Toggle, ToggleGroup, ToggleItem, ToggleMode, toggle, toggle_group,
    toggle_item,
};
pub use toolbar::{
    ActionBar, MAX_TOOLBAR_ACTIONS, MAX_TOOLBAR_GROUPS, Toolbar, ToolbarGroup, action_bar,
    toolbar, toolbar_group,
};
pub use typography::{
    Text, TextVariant, Title, Typography, TypographyContext, text, title, typography,
};
pub use visually_hidden::{MAX_VISUALLY_HIDDEN_BYTES, VisuallyHidden, visually_hidden};

pub mod prelude {
    pub use crate::{
        Accordion, AccordionItem, ActionIcon, Anchor, AnchorRequest, AspectRatio, Badge,
        BadgeVariant, Blockquote, Box, BreadcrumbItem, Breadcrumbs, Burger, Button,
        ButtonGroup, ButtonVariant, Card, Center, CheckState, Checkbox, CheckboxGroup,
        CheckboxItem, Chip, ChipGroup, ChipItem, ChipMode, CloseButton, Code, Collapsible,
        Collection, CollectionLimits, CollectionMember, ColorScheme, Container, ContentPolicy,
        ControlValue, Currency, DataList, DataListItem, Description, Dialog, Drawer,
        DrawerSide, EmptyState, Field, FieldControl, FieldError, FieldOptions, Fieldset, Flex,
        Form, FormButton, FormEntry, FormError, FormField, FormFieldState, FormLimits,
        FormState, FormStatus, Grid, GridItem, Group, Highlight, Hue, InputAddon, InputGroup,
        InputSection, Item, Kbd, KbdKey, KbdPlatform, Label, Mark, MixedCheckbox, MountedItem,
        NavLink, NumberFormat, NumberFormatter, NumberLocale, PageItem, Pagination, Paper,
        PasswordInput, Pill, Popover, PopoverAlign, Radio, RadioGroup, RovingFocus, Segment,
        SegmentedControl, Separator, ShadowSize, SimpleGrid, Size, Space, Spoiler, Stack,
        SubmissionLimits, SubmitStatus, SubmitToken, Switch, Tab, TabActivation, Tabs, Text,
        TextField, TextList, TextListItem, TextVariant, Textarea, Theme, Timeline,
        TimelineItem, TimelineProgress, Title, Toggle, ToggleGroup, ToggleItem, ToggleMode,
        Typeahead, TypeaheadConfig, Typography, TypographyContext, ValidationErrors,
        ValidationFuture, accordion, accordion_item, action_icon, anchor, aspect_ratio, badge,
        blockquote, box_, breadcrumb_item, breadcrumbs, burger, button, button_group, card,
        center, checkbox, checkbox_group, checkbox_item, chip, chip_group, chip_item,
        close_button, code, collapsible, container, data_list, data_list_item, description,
        dialog, drawer, empty_state, field, field_description, field_error, fieldset, flex,
        form, grid, grid_item, group, highlight, input_addon, input_group, input_section, item,
        kbd, label, mark, mixed_checkbox, nav_link, number_formatter, pagination,
        pagination_window, paper, password_input, pill, popover, radio, radio_group,
        reset_button, segment, segmented_control, separator, simple_grid, space, spoiler,
        stack, submit_button, switch, tab, tabs, text, text_field, text_list, text_list_item,
        textarea, timeline, timeline_item, title, toggle, toggle_group, toggle_item,
        typography,
    };
    pub use crate::{ActionBar, Toolbar, ToolbarGroup, action_bar, toolbar, toolbar_group};
    pub use crate::{
        Affix, AffixContainer, AffixContext, AffixHorizontal, AffixOffset, AffixVertical,
        affix, affix_container,
    };
    pub use crate::{
        AppShell, ShellContext, ShellLayout, ShellMode, Sidebar, SidebarContext, SidebarHandle,
        app_shell, sidebar,
    };
    pub use crate::{
        Autocomplete, DrawerSnap, DrawerSnaps, MultiSelect, MultiSelectHandle, autocomplete,
        autocomplete_option, multi_select,
    };
    pub use crate::{
        Avatar, AvatarGroup, AvatarShape, DropdownMenu, Menu, MenuEntry, MenuItem,
        MenuRadioGroup, MenuRadioItem, MenuSubmenu, VisuallyHidden, avatar, avatar_group,
        dropdown_menu, menu, menu_checkbox, menu_item, menu_radio_group, menu_radio_item,
        menu_submenu, visually_hidden,
    };
    pub use crate::{ColorSwatch, color_swatch};
    pub use crate::{
        Combobox, ComboboxFilter, LoadingOverlay, MAX_COMBOBOX_SEARCH_BYTES, Overlay, combobox,
        loading_overlay, overlay,
    };
    pub use crate::{
        MAX_BREADCRUMB_ITEMS, MAX_PAGINATION_WINDOW_ITEMS, MAX_STEPPER_STEPS, Step, StepStatus,
        Stepper, step, stepper,
    };
    pub use crate::{
        MAX_PENDING_MODALS, ModalOutcome, ModalRequest, ModalToken, ModalsHandle,
        ModalsManager, Select, confirmation_request, modal_request, modals_manager, select,
    };
    pub use crate::{
        MAX_PENDING_SCROLL_INTENTS, ScrollArea, ScrollContext, ScrollHandle, Scrollbars,
        Scroller, scroll_area, scroller,
    };
    pub use crate::{
        NavigationFocusable, NavigationMenu, NavigationMenuContext, NavigationMenuLink,
        NavigationMenuPanel, navigation_menu, navigation_menu_link, navigation_menu_panel,
    };
    pub use crate::{
        NavigationProgress, NavigationProgressHandle, NavigationProgressSnapshot,
        NavigationProgressToken, navigation_progress,
    };
    pub use crate::{
        Notification, NotificationClock, NotificationCloseReason, NotificationRequest,
        NotificationToken, NotificationsHandle, Toaster, notification, notification_request,
        notifications, toaster,
    };
    pub use crate::{Slider, slider};
    pub use crate::{Splitter, splitter};
}

pub mod navigation_menu;
pub use navigation_menu::*;

pub mod slider;
pub use slider::{Slider, slider};

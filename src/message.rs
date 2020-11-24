use crate::gui::lang::model::{Description, RequestClicks};

#[derive(Debug, Clone)]
pub enum MainMessage {
    ToStartup,
    ToExit,
    ToWorld,
    ExitRequested,
    StartupToZone {
        server_ip: String,
        server_port: u16,
    },
    DescriptionToZone {
        request_clicks: Option<RequestClicks>,
    },
    NewCharacterId {
        character_id: String,
    },
    ToDescriptionWithDescription {
        description: Description,
        back_url: Option<String>,
    },
    ToDescriptionWithUrl {
        url: String,
        back_url: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    ConfirmButtonPressed,
    CancelButtonPressed,
    ResetProgressBar,
    LocalServerPressed,
    S2BuxFrServerPressed,
    TextInputSelected(i32),
    SubmitButtonPressed,
    CheckBoxChecked(i32),
    CheckBoxUnchecked(i32),
    LinkButtonPressed(i32),
    LinkButtonReleased(String),
    GroupLinkButtonPressed(i32),
    GroupLinkButtonReleased(String),
    ChoicePressed(i32, i32),
    SearchByStrInputPressed(i32),
    SearchByStrButtonPressed(i32, i32),
    SearchByStrButtonReleased(i32, i32),
    GoBackFromGroupButtonPressed,
    GoBackZoneButtonPressed,
    GoBackButtonPressed(String),
    WorldMenuButtonPressed,
    CardMenuButtonPressed,
    EventsMenuButtonPressed,
    BusinessMenuButtonPressed,
    AffinitiesMenuButtonPressed,
    ZoneMenuButtonPressed,
    ZoneMessagesMenuButtonPressed,
    ConversationsMenuButtonPressed,
    InventoryMenuButtonPressed,
    ActionMenuButtonPressed,
    BuildMenuButtonPressed,
    ExitMenuButtonPressed,
    ToStartupPressed,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum ProcessStage {
    #[default]
    Init,
    SwitchDetect,

    //JunOS logic
    JunosWaitForBootloader,
    JunosWaitForBootloader2,
    JunosWaitForBootloader3,
    JunosWaitForBootloader4,
    JunosEnterSingleUserMode,
    JunosAwaitBoot,
    JunosAwaitRecoveryShell,
    JunosAnswerZeroize,
    JunosAwaitZeroizeFinish,
    JunosLogin,
    JunosHappyCli,
    JunosBackupImageCli,
    JunosBackupImageCli2,
    JunosBackupImageCli3,
    JunosBackupImageCli4,
    JunosAwaitBackupImageCliRecoveryFinish,
    JunosVersionOutput,
    JunosChassisOutput,
    JunosPoweroffConfirm,
    JunosWaitForPoweroff,

    //EX2300C logic
    Junos23WaitForBootloader,
    Junos23Bootloader1,
    Junos23Bootloader2,
    Junos23AwaitRecoveryShell,
    Junos23AnswerZeroize,
    Junos23AwaitZeroizeFinish,
    Junos23AwaitZeroizeFinish2,

    //Arista logic
    AristaWaitForBootloader,
    AristaWipeStartupConfig,
    AristaRebootAfterStartupConfigWipe,
    AristaWaitForReboot,
    AristaLoggingIn,
    AristaVersionOutput,

    //End the process.
    EndJob,
}

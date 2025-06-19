id = "common_junos_wipe"

state "JunosLogin" {
  transition {
    target = "JunosHappyCli"
    trigger {
      type   = "string"
      string = "root@:RE:0%"
    }
    action {
      type = "SendLine"
      line = "echo \"y\" | crontab -r"
    }
    action {
      type = "SendLine"
      line = "rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*"
    }
    action {
      type = "SendLine"
      line = "cli"
    }
  }
  transition {
    target = "JunosHappyCli"
    trigger {
      type   = "string"
      string = "root@:LC:0%"
    }
    action {
      type = "AddDeviceInfo"
      flag = "StrangeCLIPrompt"
    }
    action {
      type = "SendLine"
      line = "echo \"y\" | crontab -r"
    }
    action {
      type = "SendLine"
      line = "rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*"
    }
    action {
      type = "SendLine"
      line = "cli"
    }
  }

  transition {
    target = "JunosHappyCli"
    trigger {
      type  = "regex"
      regex = "root@[A-Za-z0-9\\-]+:RE:0%"
    }
    action {
      type = "AddDeviceInfo"
      flag = "KeptHostname"
    }
    action {
      type = "SendLine"
      line = "echo \"y\" | crontab -r"
    }
    action {
      type = "SendLine"
      line = "rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*"
    }
    action {
      type = "SendLine"
      line = "cli"
    }
  }
  transition {
    target = "JunosHappyCli"
    trigger {
      type  = "regex"
      regex = "root@[A-Za-z0-9\\-]+:LC:0%"
    }
    action {
      type = "AddDeviceInfo"
      flag = "KeptHostname"
    }
    action {
      type = "AddDeviceInfo"
      flag = "StrangeCLIPrompt"
    }
    action {
      type = "SendLine"
      line = "echo \"y\" | crontab -r"
    }
    action {
      type = "SendLine"
      line = "rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*"
    }
    action {
      type = "SendLine"
      line = "cli"
    }
  }
  transition {
    target = "JunosBackupImageCli1"
    trigger {
      type   = "string"
      string = "Please re-install JUNOS"
    }
    action {
      type = "SendLine"
      line = "echo \"y\" | crontab -r"
    }
    action {
      type = "SendLine"
      line = "rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*"
    }
    action {
      type = "SendLine"
      line = "cli"
    }
  }
}

state "JunosHappyCli" {
  transition {
    target = "JunosHappyCli"
    trigger {
      type = "string"
      string = "Retry connection attempts ? [yes,no] (yes)"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
  transition {
    target = "JunosVersionOutput"
    trigger {
      type   = "string"
      string = "root>"
    }
    action {
      type = "SendLine"
      line = "show version | no-more"
    }
  }
  transition {
    target = "JunosVersionOutput"
    trigger {
      type   = "regex"
      regex = "root@[A-Za-z0-9\\-]+>"
    }
    action {
      type = "AddDeviceInfo"
      flag = "KeptHostname"
    }
    action {
      type = "SendLine"
      line = "show version | no-more"
    }
  }
}

state "JunosBackupImageCli1" {
  transition {
    target = "JunosBackupImageCli2"
    trigger {
      type   = "string"
      string = "root>"
    }
    action {
      type = "AddDeviceInfo"
      flag = "AlternateImage"
    }
    action {
      type = "SendLine"
      line = "request system snapshot media internal slice alternate"
    }
  }
  transition {
    target = "JunosBackupImageCli2"
    trigger {
      type   = "regex"
      regex = "root@[A-Za-z0-9\\-]+>"
    }
    action {
      type = "AddDeviceInfo"
      flag = "KeptHostname"
    }
    action {
      type = "AddDeviceInfo"
      flag = "AlternateImage"
    }
    action {
      type = "SendLine"
      line = "request system snapshot media internal slice alternate"
    }
  }
}

state "JunosBackupImageCli2" {
  transition {
    target = "JunosBackupImageCli3"
    trigger {
      type   = "string"
      string = "[yes,no] (no)"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
  transition {
    target = "JunosBackupImageCli3"
    trigger {
      type   = "string"
      string = "The following filesystems were archived:"
    }
  }
}

state "JunosBackupImageCli3" {
  transition {
    target = "JunosBackupImageCli4"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?>"
    }
    action {
      type = "SendLine"
      line = "request system reboot slice alternate media internal at now"
    }
  }
}

state "JunosBackupImageCli4" {
  transition {
    target = "JunosAwaitBackupImageCliRecoveryFinish"
    trigger {
      type   = "string"
      string = "[yes,no] (no)"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
}

state "JunosAwaitBackupImageCliRecoveryFinish" {
  transition {
    target = "JunosLogin"
    trigger {
      type   = "string"
      string = "login:"
    }
    action {
      type = "SendLine"
      line = "root"
    }
  }
}

state "JunosVersionOutput" {
  transition {
    target = "JunosChassisOutput"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?>"
    }
    action {
      type = "Function"
      func = "CaptureJunosVersion"
    }
    action {
      type = "SendLine"
      line = "show chassis hardware | no-more"
    }
  }
}

state "JunosChassisOutput" {
  transition {
    target = "HookJunosCLI"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?>"
    }
    action {
      type = "Function"
      func = "CaptureChassisOutput"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "HookJunosCLI" {
  transition {
    target = "JunosPoweroff"
    trigger {
      type = "immediate"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "JunosPoweroff" {
  transition {
    target = "JunosPoweroffConfirm"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?>"
    }
    action {
      type = "SendLine"
      line = "request system power-off at now"
    }
  }
}

state "JunosPoweroffConfirm" {
  transition {
    target = "JunosWaitForPoweroff"
    trigger {
      type   = "string"
      string = "[yes,no] (no)"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
  transition {
    target = "JunosPoweroffConfirm"
    trigger {
      type   = "string"
      string = "error: command is not valid"
    }
    action {
      type = "SendLine"
      line = "request system halt at now"
    }
  }
}

state "JunosWaitForPoweroff" {
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "has halted."
    }
  }
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "acpi0: Powering system off"
    }
  }
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "Rebooting..."
    }
  }
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "reboot: Power down"
    }
  }
}

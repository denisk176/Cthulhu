id = "modern_junos_wipe"

depends = [
  "common_junos_wipe"
]

state "SwitchDetect" {
  merge = "append"
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "Booting from Flash A"
    }
    action {
      type   = "UpdatePortStatus"
      status = "Busy"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "Primary BIOS version CDEN_P_EX1"
    }
    action {
      type   = "UpdatePortStatus"
      status = "Busy"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "U-Boot 2016"
    }
    action {
      type   = "UpdatePortStatus"
      status = "Busy"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "Juniper U-Boot Script File"
    }
    action {
      type   = "UpdatePortStatus"
      status = "Busy"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "U-Boot 2021"
    }
    action {
      type   = "UpdatePortStatus"
      status = "Busy"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "Juniper Linux"
    }
    action {
      type   = "UpdatePortStatus"
      status = "Busy"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
}

state "ModernJunosWaitForBootloader" {
  transition {
    target = "ModernJunosBootloader1"
    trigger {
      type = "string"
      string = "seconds... (press Ctrl-C to interrupt)"
    }
    action {
      type = "SendControl"
      char = "c"
    }
  }
}

state "ModernJunosBootloader1" {
  transition {
    target = "ModernJunosBootloader2"
    trigger {
      type = "string"
      string = "Choice:"
    }
    action {
      type = "Send"
      text = "5"
    }
    action {
      type = "Flush"
    }
  }
}

state "ModernJunosBootloader2" {
  transition {
    target = "ModernJunosAwaitRecoveryShell"
    trigger {
      type = "string"
      string = "Choice:"
    }
    action {
      type = "Send"
      text = "2"
    }
    action {
      type = "Flush"
    }
  }
}

state "ModernJunosAwaitRecoveryShell" {
  transition {
    target = "ModernJunosAnswerZeroize"
    trigger {
      type = "regex"
      regex = "\\{[a-z0-9]+:0\\}"
    }
    action {
      type = "SendLine"
      line = "request system zeroize"
    }
  }
}

state "ModernJunosAnswerZeroize" {
  transition {
    target = "ModernJunosAwaitZeroizeFinish"
    trigger {
      type = "string"
      string = "Erase all data, including configuration and log files?"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
}

state "ModernJunosAwaitZeroizeFinish" {
  transition {
    target = "ModernJunosAwaitZeroizeFinish2"
    trigger {
      type = "regex"
      regex = "FreeBSD/[AaipP]"
    }
  }
}

state "ModernJunosAwaitZeroizeFinish2" {
  transition {
    target = "JunosLogin"
    trigger {
      type = "string"
      string = "login:"
    }
    action {
      type = "SendLine"
      line = "root"
    }
  }
}

id = "arista_wipe"

state "SwitchDetect" {
  merge = "append"
  transition {
    target = "AristaWaitForBootloader"
    trigger {
      type   = "string"
      string = "Aboot"
    }
    action {
      type   = "UpdatePortStatus"
      status = "Busy"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Arista"
    }
  }
}

state "AristaWaitForBootloader" {
  transition {
    target = "AristaWipeStartupConfig"
    trigger {
      type = "string"
      string = "Press Control-C now to enter Aboot shell"
    }
    action {
      type = "SendControl"
      char = "c"
    }
  }
}

state "AristaWipeStartupConfig" {
  transition {
    target = "AristaRebootAfterStartupConfigWipe"
    trigger {
      type = "string"
      string = "Aboot#"
    }
    action {
      type = "SendLine"
      line = "rm -r /mnt/flash/.persist /mnt/flash/startup-config"
    }
  }
}

state "AristaRebootAfterStartupConfigWipe" {
  transition {
    target = "AristaWaitForReboot"
    trigger {
      type = "string"
      string = "Aboot#"
    }
    action {
      type = "SendLine"
      line = "exit"
    }
  }
}

state "AristaWaitForReboot" {
  transition {
    target = "AristaLoggingIn"
    trigger {
      type = "string"
      string = "login:"
    }
    action {
      type = "SendLine"
      line = "admin"
    }
  }
}

state "AristaLoggingIn" {
  transition {
    target = "AristaVersionOutput"
    trigger {
      type = "string"
      string = "localhost>"
    }
    action {
      type = "SendLine"
      line = "show version"
    }
  }
}

state "AristaVersionOutput" {
  transition {
    target = "AristaEnable"
    trigger {
      type = "string"
      string = "localhost>"
    }
    action {
      type = "Function"
      func = "CaptureAristaVersion"
    }
    action {
      type = "SendLine"
      line = "enable"
    }
  }
}

state "AristaEnable" {
  transition {
    target = "AristaEraseCores"
    trigger {
      type = "string"
      string = "localhost#"
    }
    action {
      type = "SendLine"
      line = "bash rm -rfv /var/core/core.*"
    }
  }
}

state "AristaEraseCores" {
  transition {
    target = "HookAristaCLI"
    trigger {
      type = "string"
      string = "localhost#"
    }
  }
}

state "HookAristaCLI" {
  transition {
    target = "EndJob"
    trigger {
      type = "immediate"
    }
    action {
      type = "SendLine"
      line = "exit"
    }
  }
}

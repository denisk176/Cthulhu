id = "aruba_wipe"

state "SwitchDetect" {
  merge = "append"
  transition {
    target = "ArubaWaitForBootloader"
    trigger {
      type   = "string"
      string = "APBoot"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Aruba"
    }
  }
}

state "ArubaWaitForBootloader" {
  transition {
    target = "ArubaWaitForBootPrompt"
    trigger {
      type   = "string"
      string = "Hit <Enter> to stop autoboot"
    }
    action {
      type = "Function"
      func = "CaptureArubaAPModel"
    }
    action {
      type = "SendLine"
      line = ""
    }
    action {
      type = "Flush"
    }
  }
}

state "ArubaWaitForBootPrompt" {
  transition {
    target = "ArubaWaitForWipe"
    trigger {
      type   = "string"
      string = "apboot>"
    }
    action {
      type = "SendLine"
      line = "factory_reset"
    }
  }
}

state "ArubaWaitForWipe" {
  transition {
    target = "ArubaWaitForSerial"
    trigger {
      type   = "string"
      string = "apboot>"
    }
    action {
      type = "SendLine"
      line = "mfginfo"
    }
  }
}

state "ArubaWaitForSerial" {
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "apboot>"
    }
    action {
      type = "Function"
      func = "CaptureArubaAPSerial"
    }
    action {
      type = "SendLine"
      line = "reset"
    }
  }
}


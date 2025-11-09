id = "vmhost_junos_wipe"

depends = [
  "common_junos_wipe"
]

state "SwitchDetect" {
  merge = "append"

  # QFX5100
  transition {
    target = "VMHostJunosLoader1"
    trigger {
      type = "string"
      string = "deH2O version : V0018.1"
    }
  }
  transition {
    target = "VMHostJunosLoader1"
    trigger {
      type = "string"
      string = "GNU GRUB  version 0.97"
    }
  }
}

state "VMHostJunosLoader1" {
  transition {
    target = "VMHostJunosLoader2"
    trigger {
      type   = "string"
      string = "The highlighted entry will be booted automatically"
    }
    action {
      type = "SendControl"
      char = "n"
    }
    action {
      type = "SendControl"
      char = "n"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "VMHostJunosLoader2" {
  transition {
    target = "VMHostJunosSelect1"
    trigger {
      type = "string"
      string = "Select an image for recovery or cancel to continue:"
    }
  }
}

state "VMHostJunosSelect1" {
  transition {
    target = "VMHostJunosSelect2"
    trigger {
      type = "string"
      string = "Pick an option"
    }
    action {
      type = "SendLine"
      line = "1"
    }
  }
}

state "VMHostJunosSelect2" {
  transition {
    target = "VMHostJunosSelect3"
    trigger {
      type = "string"
      string = "Please confirm[y/n]"
    }
    action {
      type = "SendLine"
      line = "y"
    }
  }
}

state "VMHostJunosSelect3" {
  transition {
    target = "VMHostJunosSelect4"
    trigger {
      type = "string"
      string = "Do you want to format Junos data disk?[y/n]"
    }
    action {
      type = "SendLine"
      line = "y"
    }
  }
}

state "VMHostJunosSelect4" {
  transition {
    target = "VMHostJunosWaitFormat"
    trigger {
      type = "string"
      string = "Do you want to format Junos config disk?[y/n]"
    }
    action {
      type = "SendLine"
      line = "y"
    }
  }
}

state "VMHostJunosWaitFormat" {
  transition {
    target = "VMHostJunosWaitBoot"
    trigger {
      type   = "regex"
      regex = "FreeBSD/.* bootstrap loader"
    }
  }
}

state "VMHostJunosWaitBoot" {
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

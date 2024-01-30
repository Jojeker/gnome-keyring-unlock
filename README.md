# Gnome Keyring Unlock

Unlock the gnome keyring from a key file.
It is useful if you have the password on a USB drive, and want to unlock the keyring automatically.


## Enable the service for your user

```bash
ln -s /usr/lib/systemd/system/gkd-unlock.service ~/.config/systemd/user/
systemctl --user enable gnome-keyring-unlock.service
```

After a reboot, the keyring should be unlocked automatically.
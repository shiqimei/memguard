# memguard

```
cargo run --release -- --max-memory 24GB --interval 10
```

## Service

```
sudo cp memguard.service /etc/systemd/system/
sudo systemctl enable memguard.service
sudo systemctl start memguard.service
sudo systemctl status memguard.service
```

```
» sudo systemctl status memguard.service
● memguard.service - Memory Guard Service
     Loaded: loaded (/etc/systemd/system/memguard.service; enabled; preset: enabled)
     Active: active (running) since Fri 2024-12-20 05:22:21 UTC; 4s ago
   Main PID: 109651 (sudo)
      Tasks: 10 (limit: 75746)
     Memory: 11.1M (peak: 13.1M)
        CPU: 195ms
     CGroup: /system.slice/memguard.service
             ├─109651 /usr/bin/sudo /usr/local/bin/memguard --max-memory=16GB
             └─109653 /usr/local/bin/memguard --max-memory=16GB

Dec 20 05:22:21 ip-172-31-45-249 systemd[1]: Started memguard.service - Memory Guard Service.
Dec 20 05:22:21 ip-172-31-45-249 sudo[109651]:     root : PWD=/ ; USER=root ; COMMAND=/usr/local/bin/memguard --max-mem>
Dec 20 05:22:21 ip-172-31-45-249 sudo[109651]: pam_unix(sudo:session): session opened for user root(uid=0) by (uid=0)
lines 1-14/14 (END)
```
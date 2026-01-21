if not exist "C:\Users\Admin\.ssh" mkdir "C:\Users\Admin\.ssh"
copy /Y key_v3 "C:\Users\Admin\.ssh\id_ed25519"
copy /Y key_v3.pub "C:\Users\Admin\.ssh\id_ed25519.pub"

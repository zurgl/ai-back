# Using ION as a file server

## Some instruction

- add entry to `/etc/hosts`
- connect dashboard
- grab root pass

```bash
apt update
apt upgrade
reboot

apt -y install curl ca-certificates gcc libc6-dev libssl-dev \
tree curl git htop unzip most gettext \
g++ automake autoconf libtool zip pkg-config cmake \
libncurses5-dev libncursesw5-dev doxygen libtool-bin

hostnamectl set-hostname crunch
adduser zu
usermod -aG sudo zu
visudo
zu ALL=(ALL) NOPASSWD: ALL
%sudo ALL=NOPASSWD: ALL
updatedb

su - zu
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo install exa bat cargo-add sfz du-dust
cargo install cargo-udeps cargo-upgrades cargo-watch cargo-audit
cargo install ripgrep fd-find simple-http-server
cargo install --locked trunk 
```

sudo vim /etc/ssh/sshd_config
PermitRootLogin no
PasswordAuthentication no
sudo systemctl restart ssh
ssh-copy-id zu@ion

```text
~/.ssh/config
Host ion
  HostName 212.227.203.62
  Port 22
  User zu
  ForwardAgent yes
```

Add to DNS

```bash
CAA www 0 issue "www.digicert.com"
```

Add to Machine

```text
sudo cp foo.crt /usr/local/share/ca-certificates/foo.crt
sudo update-ca-certificates
```

## Dominator

Add some stuff
Must use yarn

```bash
rustup target add wasm32-unknown-unknown
wget -qO- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
nvm install lts/hydrogen
nvm use lts/hydrogen
corepack enable
corepack prepare yarn@stable --activate
```

## Additional info

```bash
rustup component add rustfmt --toolchain nightly
cargo run --release --bin main -- --model v2.1 --precision fp32 --height 1216 --width 1216 --steps 100 --prompt "a colorfull history of religion"
cargo install --git https://github.com/est31/cargo-udeps --locked
cargo-fmt
cargo +nightly udeps
cargo test -p models
cargo test -p models --test '*' -- --nocapture
cargo test -p actors --test '*' -- --nocapture
cargo test -p server --test '*' -- --nocapture
cargo test -p server --test mod
cargo test -p server --test run
cargo run --bin cli
cargo check
cargo watch -x 'run --release --bin diffuser'
cargo install tree-sitter-cli
tokio-console
tmux

virtualenv .ai-env
source ./.ai-env/bin/activate.fish
pip3 install torch torchvision torchaudio
deactivate

users
uptime
sudo zgrep sshd /var/log/auth.log* -h | grep -F 'Accepted'
sudo zgrep sshd /var/log/auth.log* | grep rhost | sed -re 's/.*rhost=([^ ]+).*/\1/' | sort -u

ssh-add ~/.ssh/id_rsa
nvim ~/.config/fish/config.fish
pgrep ssh-agent
killall ssh-agent
ssh-keygen -t rsa
ssh-add -v
ssh-add -l
ssh -vT git@github.com
git remote -v

node
set --universal nvm_default_version v18.13.0
nvm use default lts
nvm alias default lts/hydrogen
nvm list
nvm alias default lts
nvm use lts

locate distill | rg "rust_bert" | rg -v "debug"

wget --no-check-certificate https://huggingface.co/ainize/gpt-j-6B-float16/resolve/main/pytorch_model.bin

git log --name-status HEAD^..HEAD

rg "println" ./diffusion
rg "println" .
rg println ./**/*

RUST_BACKTRACE=full cargo run --release --bin diffuser
set -e DEV
pn next dev -H 0.0.0.0 -p 3000


gh repo clone zurgl/diffusion-front
gh auth login

systemctl status fail2ban.service
sudo systemctl stop fail2ban.service
sudo /etc/init.d/ssh reload
sudo cp jail.conf jail.local
head -20 jail.conf
cd /etc/fail2ban
sudo apt install fail2ban

icat ./images/frame0007.png
source ~/.config/fish/config.fish
echo 'alias icat="kitty +kitten icat' >> ~/.config/fish/config.fish
kitty +kitten icat Sonata.png
kitty +kitten icat Downloading\ kitty\ from:\ \e\[32mhttps://github.com/kovidgoyal/kitty/releases/download/v0.27.0/kitty-0.27.0-x86_64.txz\e\[m

rm -rf ron.vim/
cp -vi ron.vim/indent/ron.vim  ./indent/
cp -vi ron.vim/syntax/ron.vim  ./syntax/
cp -vi ron.vim/syntax/  ./syntax/
cp -vi ron.vim/ftplugin/ron.vim  ./ftplugin/
cp ron.vim/ftdetect/ron.vim  ./ftdetect/
mkdir syntax
mkdir indent
mkdir ftplugin
gh repo clone ron-rs/ron.vim
cd ~/.config/nvim

set -lx LD_LIBRARY_PATH /home/zu/rust/diffuz/target/release/build/torch-sys-2beca46c056cf73a/out/libtorch/libtorch/lib
ldd ./diffuz
sudo ldconfig
set -x LIBTORCH_LIB $LIBTORCH
set -lx LIBTORCH_INCLUDE $LIBTORCH
set -lx LD_LIBRARY_PATH $LIBTORCH/lib:$LD_LIBRARY_PATH
set -lx LIBTORCH "/home/zu/rust/diffuz/target/release/build/torch-sys-2beca46c056cf73a/out/libtorch/libtorch"
cd /home/zu/rust/diffuz/target/release/build/torch-sys-2beca46c056cf73a/out/libtorch/libtorch
locate libtorch_cuda_cu.so
ldd ./target/release/diffuz

fd png | fzf
fd png
fd *.png

mv lemonade .local/bin/
rm lemonade_linux_amd64.tar.gz
tar xvzf lemonade_linux_amd64.tar.gz
wget https://github.com/lemonade-command/lemonade/releases/download/v1.1.1/lemonade_linux_amd64.tar.gz

sudo ln -s /usr/local/bin/fish /usr/bin/fish
ln -s /usr/local/bin/fish /usr/bin/fish
ln -s /usr/bin/fish /usr/local/bin/fish

nvim .bashrc
fisher install jorgebucaran/nvm.fish
curl -sL https://git.io/fisher | source && fisher install jorgebucaran/fisher

nvim .gitconfig
git config --global user.name "zurgl"
git config --global user.email "elayar.yacine@gmail.com"

fish_update_completions

cd completions/
cd ~/.config/fish/
curl https://raw.githubusercontent.com/oh-my-fish/oh-my-fish/master/bin/install | fish
cd rust

cp -v share/man ~/.local/share/man/
cp -v share/* ~/.local/share/man/
cp -v share/* ~/.local/share/
cp -v bin/* ~/.local/bin
cd gh-cli/
cd .local/bin/

./gh-cli/bin/gh
mv gh_2.21.2_linux_amd64/ gh-cli
cd gh_2.21.2_linux_amd64/
tar -xvzf gh_2.21.2_linux_amd64.tar.gz
wget https://github.com/cli/cli/releases/download/v2.21.2/gh_2.21.2_linux_amd64.tar.gz

huggingface-cli login
pip3 install huggingface_hub pylint pyjq jq

sudo apt -y install ninja-build gettext libtool libtool-bin autoconf automake cmake libssl-dev
sudo apt -y install gunzip bunzip2 7z tar untar most unzip curl doxygen g++ pkg-config xclip
```

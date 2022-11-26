subkey inspect --scheme ecdsa $secret
subkey inspect --scheme sr25519 "$secret//0//aura"
subkey inspect --scheme ed25519 "$secret//0//grandpa"


for i in 1 2 3 4; do seed=$(subkey inspect --scheme sr25519 "$secret//$i//aura"|grep seed|cut -d':' -f2 |sed 's/^[ \t]*//;s/[ \t]*$//'); subkey insert --keystore-path "v$i" --scheme=sr25519 --key-type=aura --suri=$seed; done
for i in 1 2 3 4; do seed=$(subkey inspect --scheme ed25519 "$secret//$i//grandpa"|grep seed|cut -d':' -f2 |sed 's/^[ \t]*//;s/[ \t]*$//'); subkey insert --keystore-path "v$i" --scheme=ed25519 --key-type=gran --suri=$seed; done
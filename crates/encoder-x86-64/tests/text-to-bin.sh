#!/usr/bin/env sh

# usage:
#
#  ./text-to-bin.sh "mov rax, rbx"
#
# echo "mov rax, rbx" | ./text-to-bin.sh
#
# echo -e "mov rax, rbx\nadd rax, 1" | ./text-to-bin.sh
#
# cat | ./text-to-bin.sh << EOF
# > mov rax, rbx
# > add rax, 1
# > EOF

tmp=$(mktemp /tmp/anasm_text_to_bin.XXXXXX)

# read instructions from command line argument
inst="$1"

# read instructions from stdin if no argument is given
if [ -z "$inst" ]; then
    inst=$(cat) # read from stdin
fi

cat > "$tmp" <<EOF
BITS 64
$inst
EOF

nasm -f bin "$tmp" -o "$tmp.bin"
xxd -p "$tmp.bin"
rm -f "$tmp" "$tmp.bin"

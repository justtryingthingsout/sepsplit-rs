#!/bin/bash

if ! command -v pzb &> /dev/null; then
    echo "pzb is not installed. Please install pzb and try again."
    exit 1
elif ! command -v img4 &> /dev/null; then
    echo "img4lib is not installed. Please install img4lib and try again."
    exit 1
fi

cd "$(dirname "$0")/testfws"

filenames=(
    Firmware/all_flash/sep-firmware.n61.RELEASE.im4p
    Firmware/all_flash/sep-firmware.n71m.RELEASE.im4p
    Firmware/all_flash/sep-firmware.d10.RELEASE.im4p
    Firmware/all_flash/sep-firmware.d10.RELEASE.im4p
    Firmware/all_flash/all_flash.d11ap.production/sep-firmware.d11.RELEASE.im4p
    Firmware/all_flash/sep-firmware.d11.RELEASE.im4p
    Firmware/all_flash/sep-firmware.d21.RELEASE.im4p
    Firmware/all_flash/sep-firmware.j97.RELEASE.im4p
    Firmware/all_flash/sep-firmware.j72b.RELEASE.im4p
    AssetData/boot/Firmware/all_flash/sep-firmware.d28.RELEASE.im4p
    AssetData/boot/Firmware/all_flash/sep-firmware.n131b.RELEASE.im4p
    AssetData/boot/Firmware/all_flash/sep-firmware.n142b.RELEASE.im4p
)
urls=(
    https://updates.cdn-apple.com/2020SpringFCS/fullrestores/061-94645/267947E5-66EE-46E5-BFA4-B92EAB568C6D/iPhone_4.7_12.4.7_16G192_Restore.ipsw
    https://updates.cdn-apple.com/2022SpringFCS/fullrestores/012-07449/CC1C0468-8FEE-468B-9574-6998C7B0683D/iPhone_4.7_15.5_19F77_Restore.ipsw
    https://updates.cdn-apple.com/2020SummerFCS/fullrestores/001-46745/3B2F50D2-6AA3-4048-A679-CB422D7FD773/iPhone_4.7_P3_14.0_18A373_Restore.ipsw
    https://updates.cdn-apple.com/2021FallFCS/fullrestores/002-02953/B1EB01B7-DE28-4975-B294-49BAE9599E72/iPhone_4.7_P3_15.0_19A346_Restore.ipsw
    http://appldnld.apple.com/ios10.0/031-76519-20160907-1703BE42-71FE-11E6-AE6D-1AB934D2D062/iPhone9,2_10.0.1_14A403_Restore.ipsw
    http://appldnld.apple.com/ios11.0/091-31712-201700919-35AFA8AE-9027-11E7-9A28-2D35B64D2808/iPhone_7Plus_11.0_15A372_Restore.ipsw
    https://updates.cdn-apple.com/2021SpringFCS/fullrestores/071-17718/AD962F87-1645-410A-8BED-659CBB0449EB/iPhone_5.5_P3_14.5_18E199_Restore.ipsw
    http://updates-http.cdn-apple.com/2019FallFCS/fullrestores/091-99503/DCBD0A9C-D986-11E9-BB0A-9520DCDBD6A3/iPad_64bit_TouchID_13.1_17A844_Restore.ipsw
    https://updates.cdn-apple.com/2022SummerSeed/fullrestores/012-26209/175A6A7C-6E52-438A-8326-FC9DFEBB4E20/iPad_64bit_TouchID_ASTC_16.0_20A5303i_Restore.ipsw
    https://updates.cdn-apple.com/2023SummerSeed/patches/032-94921/A666013C-65CE-4569-9F17-523F14CB4996/com_apple_MobileAsset_SoftwareUpdate/10473dde19d3576f6ad4871d30cd6e7741704772.zip
    https://updates.cdn-apple.com/2021FallSeed/patches/002-12408/BE15E728-BFA4-4C90-800D-FB80F6CA06B9/com_apple_MobileAsset_SoftwareUpdate/efc9a3a046591663cea13eed0cbc551b7d17e85d.zip
    https://updates.cdn-apple.com/2020FallSeed/patches/001-36676/2CDB0C3C-1803-4627-B7CB-660BFD3A756D/com_apple_MobileAsset_SoftwareUpdate/63531acd54d6e802d13a0340e98164deef4ccb29.zip
)
outputfns=(
    sepfw.N61.16G192.bin
    sepfw.N71m.19F77.bin
    sepfw.D10.18A373.bin
    sepfw.D10.19A346.bin
    sepfw.D11.14A403.bin
    sepfw.D11.15A372.bin
    sepfw.D21.18E199.bin
    sepfw.J97.17A844.bin
    sepfw.J72b.20A5303i.bin
    sepfw.D28.21A5248v.bin
    sepfw.N131b.19R5559e.bin
    sepfw.N142b.18R5552f.bin
)
keys=(
    e506d463ee17ddc8fb89d3c28403a3a1e47acc11e3b512bc9d48fea2964bbde7fd900634fd15d727cc951159e853b1a8
    dae15b9f4a91cb9c1b09d1d74bb74f975db36638dfa6375cfd03babe8924fd22dfa75c1de02325c77b54fddf138a5070
    284cb005edfa5211c82757336d3b0ec204cefa29f1d0fc224a0b16ad46fd21153ca6d3779728ea76c59c5f11ed9fcd20
    bbcffc762ce33e8a7382367bd0cfb6743a38168060514aad3e367f9b334c15b15c1928a206846358640df0cfe068d976
    766926ce937061f53efd80bc2b19130006eba266cf7949a83a8bea36a690b9766a0a8f9a6366ce820f0a6178763e664d
    960e384bf2269475a8f0f24564ab3ba4c9803fe73272ed9586c82828921841df2294a13f9a3adde2fd4cd754d20179f0
    4d61659d019cc35ed7558dd565974ee823e5b3cacce549201403029f696168b531cf44b6754c848562b62626cd59b4cb
    ed10559a7fbd6b083d18d7250df06a80c094f934a1acf8361bcc793d3d9ab8ed9db10d15c8271cb7d592380fb5a62c92
    9b9ac127b6615af59136ecd02c3f79635ebe7473676cd25b15fde46d0685d2607e48e14816727735f77d7bde4e37f8dd
    acd4c21dae3f96cd64adf722921e3cce6c1744f23ee0491b8289c8341d9114c25af670c698630b4ac807413eee8702c5
    cb9076b542287eb5f20cd40dd8dc1b471fc06050e221491c61ad3be717efd95a2f4589a3c3bf77ed7a5ce9a8e79c25a9
    668a4ec73c4b8c6f35e57c0815476bf4f593d248c232f16de2d0957cf9641c3c049028e37687939db35dee3766483658
)

for i in {0..10}; do
    pzb -g "${filenames[$i]}" "${urls[$i]}"
    img4 -i "$(basename ${filenames[$i]})" -k "${keys[$i]}" -o "${outputfns[$i]}" > /dev/null
    rm "$(basename ${filenames[$i]})"
done
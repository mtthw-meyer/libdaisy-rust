#!/usr/bin/python3

def get_hex_list(hex_str):
    print('0x', hex_str)
    number =  int(hex_str, 16)
    out = '{0:=32b}'.format(number)
    return list(out[::-1])

def bytes_to_word(hex_bytes):
    hex = list(hex)
    hex.reverse()
    hex_str = ''.join([h[2:] for h in hex])
    return hex_str


def hex_to_off(*hexes):
    if len(hexes) == 8:
        hex_list1 = get_hex_list(bytes_to_word(hexes[:4]))
        hex_list2 = get_hex_list(bytes_to_word(hexes[4:]))
    else:
        hex_list1 = get_hex_list(hexes[0][2:])
        hex_list2 = get_hex_list(hexes[1][2:])

    for i in range(len(hex_list1)):
            print('{}: {} {}'.format(i, hex_list1[i], hex_list2[i]))


if __name__ == '__main__':
    import sys
    hex_to_off(*sys.argv[1:])

#!/usr/bin/env python3
from smbus import SMBus
import time
import array

i2cbus = SMBus(0)  # Create a new I2C bus
i2caddress = 0x68  # Address of MCP23017 device

def main():
    try:
#        powerOnImu()
#        time.sleep(1)
        writeSampleRateDivider(12)
#        print("")
        enableFifo()
        while True:
            printFifoContents()
    except:
        raise

def powerOnImu():
    print()


def writeSampleRateDivider(div):
    i2cbus.write_byte_data(i2caddress, 25, div)

def getFifoCount():
    high_byte = i2cbus.read_byte_data(i2caddress, 114)
    low_byte = i2cbus.read_byte_data(i2caddress, 115)
    return (high_byte << 8) | low_byte

def enableFifo():
    i2cbus.write_byte_data(i2caddress, 26, 0b01000110) # Config
    i2cbus.write_byte_data(i2caddress, 27, 0b00000000) # Gyro config
    i2cbus.write_byte_data(i2caddress, 29, 0b00000110) # Acc config  
    i2cbus.write_byte_data(i2caddress, 30, 0b00000000) # Acc config  
    
    i2cbus.write_byte_data(i2caddress, 106, 0b11001100) 
    i2cbus.write_byte_data(i2caddress, 35, 0b01111000)

def getDeviceId():
    raw_byte = i2cbus.read_byte_data(i2caddress, 0x75)
    id = (raw_byte >> 1) & int('00111111', 2)
    return id

def getTemperature():
    high_byte = i2cbus.read_byte_data(i2caddress, 65)
    low_byte = i2cbus.read_byte_data(i2caddress, 66)
    raw_temp =  (high_byte << 8) | low_byte
    return raw_temp/321 + 21

def combine(high_byte, low_byte):
    value = (high_byte << 8) | low_byte
    if value > 32767:
        value -= 65536
    return value

def printFifoContents():
    while True:
        fifo_count = getFifoCount()
        if fifo_count >= 12:
#            if fifo_count > 400:
#                print("fifo_count > 400")
            block_size = 12
            vals = array.array('i',[])
            for i in range(0, block_size):
                vals.append(i2cbus.read_byte_data(i2caddress, 116))
            acc_x = combine(vals[0], vals[1])
            acc_y = combine(vals[2], vals[3])
            acc_z = -combine(vals[4], vals[5])
#            temp = combine(vals[6], vals[7]) / 321 + 21
            gyro_x = combine(vals[6], vals[7])
            gyro_y = combine(vals[8], vals[9])
            gyro_z = combine(vals[10], vals[11])
            
            print(f"X:{acc_x}, Y:{acc_y}, Z:{acc_z}, Xg:{gyro_x}   ", end='\r')
        else:
            break

if __name__ == "__main__":
    main()
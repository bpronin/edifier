Original document: 
https://github.com/wh201906/mEDIFIER/blob/main/doc/KnownCommands/W820NB_Double_Gold.txt

## Service UUID:
    EDF00000-EDFE-DFED-FEDF-EDFEDFEDFEDF

## Edifier APP Logcat ID:
    SPPUtil

---

## Device information

### Get MAC address
    send: AA 01 C8 21 8C  
    read: BB 07 C8 xx xx xx xx xx xx yy yy

### Get firmware version
    send: AA 01 C6 21 8A  
    read: BB 04 C6 03 00 02 21 A3

### Get device fingerprint
    send: AA 01 D8 21 9C  
    read: BB 16 D8 xx .. yy yy

### Get battery level
    send: AA 01 D0 21 94  
    read: BB 02 D0 xx yy yy

### Get playback Status?
    send: AA 01 C3 21 87  
    read: BB 02 C3 03 21 9C

## Noise cancellation

### Set 'Noise cancellation off'
    send: AA 02 C1 01 21 87  
    read: BB 03 C1 01 06 21 9F

### Set 'Noise cancellation'
    send: AA 02 C1 02 21 88      
    read: BB 03 C1 02 06 21 A0

### Set 'Ambient sound'
    send: AA 02 C1 03 21 89  
    read: BB 03 C1 03 06 21 A1

### Set 'Ambient sound' with volume
**+3**  

    send: AA 03 C1 03 09 21 93  
    read: BB 03 C1 03 09 21 A4

**+2**  

    send: AA 03 C1 03 08 21 92  
    read: BB 03 C1 03 08 21 A3

**+1**  

    send: AA 03 C1 03 07 21 91  
    read: BB 03 C1 03 07 21 A2

**0**  

    send: AA 03 C1 03 06 21 90  
    read: BB 03 C1 03 06 21 A1

**-1**  

    send: AA 03 C1 03 05 21 8F  
    read: BB 03 C1 03 05 21 A0

**-2**  

    send: AA 03 C1 03 04 21 8E  
    read: BB 03 C1 03 04 21 9F

**-3**  

    send: AA 03 C1 03 03 21 8D  
    read: BB 03 C1 03 03 21 9E

## Button control set

### Get button control set
    send: AA 02 F0 0A 21 BF  
    read: BB 03 F0 0A xx yy yy

### Set 'Noise cancellation / Noise cancellation off / Ambient sound' (All)
    send: AA 03 F1 0A 07 21 C8  
    read: BB 03 F1 0A 07 21 D9

### Set 'Noise cancellation / Ambient sound'
    send: AA 03 F1 0A 06 21 C7  
    read: BB 03 F1 0A 06 21 D8

### Set 'Noise cancellation off / Ambient sound'
    send: AA 03 F1 0A 05 21 C6  
    read: BB 03 F1 0A 05 21 D7

### Set 'Noise cancellation / Noise cancellation off'
    send: AA 03 F1 0A 03 21 C4  
    read: BB 03 F1 0A 03 21 D5

## Prompt volume

### Get prompt volume
    send: AA 01 05 20 C9  
    read: BB 02 05 xx yy yy

### Set prompt volume
    send: AA 02 06 xx yy yy  
    read: BB 02 06 xx yy yy

    xx = 0..15  

## Device control

### Power off
    send: AA 01 CE 21 92
    read: none

### Disconnect bluetooth
    send: AA 01 CD 21 91
    read: none

### Re-pair
    send: AA 01 CF 21 93
    read: none

### Reset factory defaults
    send: AA 01 07 20 CB
    read: none

## Auto power-off

### Get auto power-off time

**5 min**

    send: AA 01 D3 21 97  
    read: BB 03 D3 00 05 21 AF

**disabled**

    send: AA 01 D3 21 97  
    read: BB 03 D3 00 21 AF

### Disable auto power-off
    send: AA 01 D2 21 96  
    read: CC 02 D2 01 21 BA

### Set auto power-off time

**5 min**  

    send: AA 03 D1 00 05 21 9C  
    read: CC 02 D1 01 21 B9

**3 hour (180 min)**    

    send: AA 03 D1 00 B4 22 4B  
    read: CC 02 D1 01 21 B9

## Game Mode

### Get Game Mode Status
    send: AA 01 08 20 CC  
    read: BB 02 08 xx yy yy

### Set Game Mode On
    send: AA 02 09 01 20 CF  
    read: BB 02 09 01 20 E0

### Set Game Mode Off
    send: AA 02 09 00 20 CE  
    read: BB 02 09 00 20 DF

## EQ mode

### Get EQ mode
    send: AA 01 D5 21 99  
    read: BB 02 D5 xx yy yy

### Set 'Classic' EQ mode
    send: AA 02 C4 00 21 89  
    read: CC 02 C4 00 21 AB  

### Set 'Pop' EQ mode
    send: AA 02 C4 01 21 8A  
    read: CC 02 C4 01 21 AC  

### Set 'Classical' EQ mode
    send: AA 02 C4 02 21 8B  
    read: CC 02 C4 02 21 AD  

### Set 'Rock' EQ mode
    send: AA 02 C4 03 21 8C  
    read: CC 02 C4 03 21 AE  

## Device name

### Get name
    send: AA 01 C9 21 8D  
    read: BB 19 C9 xx .. yy yy

### Set name
**abc**  

    send: AA 04 CA 61 62 63 22 B7  
    read: CC 02 CA 01 21 B2

**12345**  

    send: AA 06 CA 31 32 33 34 35 22 92  
    read: CC 02 CA 01 21 B2

## LDAC control

### Get LDAC mode
    send: AA 01 48 21 0C  
    read: BB 02 48 01 21 1F

### Set LDAC mode
**Off** 

    send: AA 02 49 00 21 0E  
    read: BB 02 49 00 21 1F

**48k**

    send: AA 02 49 01 21 0F  
    read: BB 02 49 01 21 20

**96k**  
    
    send: AA 02 49 02 21 10  
    read: BB 02 49 02 21 21

---

## Unknown and hypothetic commands:
    Now Playing ?
    
### Get?
send: AA 01 68 21 2C  
read: BB 02 68 00 213E



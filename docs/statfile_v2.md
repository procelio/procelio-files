# The StatFile File Format: Version `2`

All multi-byte numbers are big-endian and unsigned unless otherwise mentioned

### Header Region
```
4 bytes: magic number (0x1EF1A757)
4 bytes: version (2)

? bytes: "blocks" flags region
? bytes: "attacks" flags region
```
### Flags Region
```
4 bytes: the number of flagged entities

K times: each entity
  4 bytes: the entity ID#
  1 byte: the number of flags
  N times: each flagged value
    1 byte: the flag 
    4 bytes SIGNED: the flag's value
```

### JSON
Check src/files/statfile/statfile.rs for the specifics. Input should be of form
```
{
  "blocks": {
    1: {
        "name": "cube",
        "health": 123,
        "mass": 456
    },
    2: {
        "name": "tetra",
        "health": 234,
        "mass": 567
    }
  },
  "attacks": {
      4: {
          "name": "smg_t1",
          "damage": 125
      }
  }
}
```

### Flags
There are different flags which may be set, and correspond to different values in the generated statsfile
#### Block Flags
```
"health" => 0
"mass" => 1
"cost" => 2
"roboRanking" => 3
"cpuCost" => 4
"thrust" => 5
"rotationSpeed" => 6
```

#### Attacks Flags
```
"damage" => 7
```
# The RobotFile File Format: Version `2`

All multi-byte numbers are big-endian and unsigned unless otherwise mentioned

### Header Region
```
4 bytes: magic number (0xC571B040)
4 bytes: version (3)
8 bytes: metadata

1 byte: length of bot name
 N bytes: bot name (bytes comprising UTF8 name)

4 bytes: num parts
 K parts

4 bytes: num cosmetics
 K cosmetics

16 bytes: MD5 hash from byte 8 (after #/version) to just before the hash
```

### Part
```
4 bytes: block type
1 byte SIGNED: x position
1 byte SIGNED: y position
1 byte SIGNED: z position
1 byte: rotation
1 byte: color red
1 byte: color green
1 byte: color blue
1 byte: alpha channel
1 byte: length of "extra data" region (max 64)
N bytes: "extra data" (game-defined)
```

pub struct JsonRobot {
    name: String,
    metadata: u64,
    parts: Vec<JsonPart>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonPart {
    id: u32,
    pos: [i8; 3],
    rot: u8,
    color: [u8; 3],
    alpha: u8,
    extra_data: Vec<u8>
}

### Cosmetic
```
4 bytes: cosmetic type
4 bytes: block onto which cosmetic applies
1 byte: length of "extra data" region (max 64)
N bytes: "extra data" (game-defined)
```

### JSON
Check src/files/robot/robot.rs for the specifics. Input should be of form
```
{
  "name": "robot",
  "metadata": 3,
  "parts": [
    {
      "id": 32,
      "pos": [1, 4, -1],
      "rot": 68,
      "color": [255, 255, 0],
      "alpha": 255,
      "extra_data": []
    },
    {
      "id": 32,
      "pos": [-1, 4, -1],
      "rot": 86,
      "color": [255, 255, 0],
      "alpha": 255,
      "extra_data": []
    },
    {
      "id": 57,
      "pos": [0, 4, -1],
      "rot": 0,
      "color": [0, 70, 125],
      "alpha": 255,
      "extra_data": [0, 219, 75, 6]
    }
  ],
  "cosmetics": [
    {
      "id": 432,
      "part_on": 1,
      "extra_data": []
    }
  ]
}
```
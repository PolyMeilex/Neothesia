# Customization

## How to change color schema

It is available in `setting.ron` (file created automatically after first launch of neothesia):

```
Config(
    color_schema: [
        (
            base: (93, 188, 255), // Color of white keys (RGB)
            dark: (48, 124, 255), // Color of black keys
        ),
        (
            base: (210, 89, 222), // Colors for second track...
            dark: (125, 69, 134),
        ),
    ],
)
```

### MacOS

Right click on Neothesia.app and click "Open package contents" and navigate to `Contents/Resources/setting.ron`

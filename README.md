# declutter

Add rules to your file system and spot bad files.

```
./declutter [config-path]
```
Default config path is `declutter.yaml`

### Config

Refer to `sample-fs` in repo root.

```yaml
# each top-level key is the path
sample-fs:
  recursive: false
  allow-name:
    - pngs
    - txts
  allow-type: dir
  # only allow directories and anything named "pngs" or "txts"

sample-fs/images:
  recursive: true
  allow-type:
    - .png
    - .jpg
    - .jpeg
    - .bmp
    - .svg

sample-fs/txts:
  recursive: false
  allow-type: .txt
  # only allow txt files, no directories

```

Options
* `recursive`: if yes, apply rule recursively (implies allow directories)
* `allow-type`: only allow directories / certain types of files
* `allow-name`: only allow directories / files with given names

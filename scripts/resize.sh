for f in ./screenshots/*.source.png; do
  f=${f/.source.png/""}
  ffmpeg -y -i $f.source.png -vf scale=400:-1 $f.png
done

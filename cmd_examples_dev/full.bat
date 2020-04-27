..\target\release\chrono-photo ^
  --pattern "../test_data/generated/image-*.jpg" ^
  --frames ././1 ^
  --temp-dir ../test_data/temp ^
  --output ../test_data/out.jpg ^
  --output-blend ../test_data/out-debug.png ^
  --mode outlier ^
  --threshold abs/0.05/0.2 ^
  --background first ^
  --outlier extreme ^
  --quality 98 ^
  --compression gzip/6 ^
  --slice rows/2 ^
  --sample 20 ^
  --weights 1 1 1 0 ^
  --debug
pause

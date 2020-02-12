
__kernel void aggregate(
  __write_only image2d_t out_image,
  __global float4* samples,
  __private int num_samples
) {
  int x_pos = get_global_id(0);
  int y_pos = get_global_id(1);
  int sample_offset = (x_pos * get_global_size(1) + y_pos) * num_samples;
  float4 aggregate = (float4)(0, 0, 0, 0);
  for (int i = 0; i < num_samples; i++) {
    aggregate += samples[sample_offset + i];
    samples[sample_offset + i] = (float4)(0, 0, 0, 1);
  }
  write_imagef(
    out_image,
    (int2)(x_pos, y_pos),
    (float4)(fmin(aggregate.xyz / num_samples, 1.0f), 1.0f)
  );
}

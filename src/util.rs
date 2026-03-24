// Where the main calculation codes will be placed.
// We gradually bump up the sophistication? of our localization algorithm.
// Basically we are calculating the Gaussian Distribution that represent the probable location of
// z (actual position)
//
//
// We want p(z|x) or more specifically p(z_t|x_0:t) given the sensor readings what is the current
// position?
// We don't want to calculate integrals so every iteration of this calculation process is
// destructured into identifying the mean and variance of p(z|x)
//
// TLDR;
// we want
// p(z|x) ~ N(sig | var)
// we want sig and var
// We will draw a heatmap that represents the gaussian distribution representing p(z_t|x_0:t)
// (something like this)
//
//
//
// 4 Markov Chain and Baysian Filtering
// 4.1 Sequential Measurement

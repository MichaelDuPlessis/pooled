# Pooled

Pooled is a threadpool library. It features zero external dependencies so that it can be a light package to install. The concept behind pooled is to allow the user
to create a parent threadpool which can then be used to create more specialized threadpools.

## Types of Threadpools

### SimplePool

This is a simple threadpool that allows users to submit jobs and that is about it. This is essentially a fire and forget type of thread pool.

### MapPool

This threadpool takes a function and a list of inputs and produces a list of results where the function maps the input list to the output list. This is
similar to rayons par_iter.


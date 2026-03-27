#include <linux/limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main() {
  printf("Hello from protected binary!\n");
  char command[PATH_MAX];
  sprintf(command, "ls -la /proc/%d/exe", getpid());
  system(command);
  sprintf(command, "ls -la /proc/%d/fd", getpid());
  system(command);
  return 0;
}

#include <linux/limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main(int argc, char **argv, char **envp) {
  printf("Hello from protected binary!\nCalled with argv:");
  for (int i = 0; i < argc; ++i)
    printf(" %s", argv[i]);
  printf("\n");
  printf("And environment:");
  for (char **env = envp; *env; ++env)
    printf(" %s", *env);
  printf("\n");

  char command[PATH_MAX];
  sprintf(command, "ls -la /proc/%d/exe", getpid());
  system(command);
  sprintf(command, "ls -la /proc/%d/fd", getpid());
  system(command);
  return 0;
}

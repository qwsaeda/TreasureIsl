/* ea:   0x0006A0C4 */
/* name: atexit */
/* size: 24 */
int __cdecl atexit(void (__cdecl *a1)())
{
  bool v1; // zf
  int result; // r0

  v1 = onexit((int)a1) == 0;
  result = 0;
  if ( v1 )
    return -1;
  return result;
}


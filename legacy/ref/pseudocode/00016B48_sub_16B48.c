/* ea:   0x00016B48 */
/* name: sub_16B48 */
/* size: 80 */
int sub_16B48(_DWORD *a1, ...)
{
  void *v2; // r1
  int result; // r0
  _BYTE v4[20]; // [sp+0h] [bp-24h] BYREF
  va_list varg_r1; // [sp+18h] [bp-Ch] BYREF

  va_start(varg_r1, a1);
  v2 = (void *)sub_16BD0(*a1, a1[1], (int *)varg_r1, v4);
  result = a1[1];
  if ( v2 != (void *)result )
    return sub_16B98((int)a1, v2);
  return result;
}


/* ea:   0x00013110 */
/* name: sub_13110 */
/* size: 124 */
int *sub_13110(int *a1, unsigned int a2, ...)
{
  int v4; // r3
  va_list v5; // r0
  unsigned int v7; // [sp+0h] [bp-28h] BYREF
  unsigned int varg_r2; // [sp+20h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+20h] [bp-8h]
  va_list varg_r3; // [sp+24h] [bp-4h] BYREF

  va_start(varg_r3, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(varg_r3, _DWORD);
  if ( a2 > (a1[1] - *a1) >> 1 )
    TerminateProcess((HANDLE)0x42, 0);
  v4 = *a1;
  v7 = ((a1[1] - *a1) >> 1) - a2;
  v5 = (va_list)&v7;
  if ( v7 >= varg_r2 )
    va_copy(v5, varg_r2a);
  sub_13248((int)a1, (void *)(v4 + 2 * a2), (void *)(v4 + 2 * (a2 + *(_DWORD *)v5)));
  return a1;
}


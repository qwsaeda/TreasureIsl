/* ea:   0x00060098 */
/* name: sub_60098 */
/* size: 128 */
int sub_60098(int *a1, int a2, unsigned int a3, ...)
{
  int v6; // r3
  va_list v7; // r0
  unsigned int v9; // [sp+0h] [bp-2Ch] BYREF
  unsigned int varg_r3; // [sp+28h] [bp-4h] BYREF
  va_list varg_r3a; // [sp+28h] [bp-4h]
  va_list va1; // [sp+2Ch] [bp+0h] BYREF

  va_start(va1, a3);
  va_start(varg_r3a, a3);
  varg_r3 = va_arg(va1, _DWORD);
  if ( a3 > (a1[1] - *a1) >> 1 )
    TerminateProcess((HANDLE)0x42, 0);
  v6 = *a1;
  v9 = ((a1[1] - *a1) >> 1) - a3;
  v7 = (va_list)&v9;
  if ( v9 >= varg_r3 )
    va_copy(v7, varg_r3a);
  sub_61524(a2, v6 + 2 * a3, v6 + 2 * (a3 + *(_DWORD *)v7));
  return a2;
}


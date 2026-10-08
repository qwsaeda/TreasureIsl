/* ea:   0x0001B06C */
/* name: sub_1B06C */
/* size: 68 */
int sub_1B06C(int a1, ...)
{
  int v2; // r3
  va_list varg_r1; // [sp+10h] [bp-Ch] BYREF

  va_start(varg_r1, a1);
  v2 = sub_1B0B0(a1 + 8, (int *)varg_r1);
  if ( v2 == *(_DWORD *)(a1 + 8) )
    return -1;
  else
    return *(_DWORD *)(v2 + 20);
}


/* ea:   0x00060AF8 */
/* name: sub_60AF8 */
/* size: 192 */
int sub_60AF8(int *a1, int a2, ...)
{
  int v3; // r4
  int v4; // r2
  va_list v5; // r0
  int v6; // r1
  int v7; // r0
  char v9[4]; // [sp+4h] [bp-34h] BYREF
  int v10; // [sp+8h] [bp-30h] BYREF
  int v11; // [sp+Ch] [bp-2Ch] BYREF
  char v12[24]; // [sp+10h] [bp-28h] BYREF
  unsigned int varg_r2; // [sp+30h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+30h] [bp-8h]
  va_list varg_r3; // [sp+34h] [bp-4h] BYREF

  va_start(varg_r3, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(varg_r3, _DWORD);
  v3 = *a1;
  v4 = (a1[1] - *a1) >> 1;
  if ( v4 == 0 )
    return -1;
  v10 = v4 - 1;
  va_copy(v5, varg_r2a);
  if ( varg_r2 >= v4 - 1 )
    v5 = (va_list)&v10;
  v6 = *(_DWORD *)sub_61D5C(v12, v3 + 2 * (*(_DWORD *)v5 + 1), v3, a2, v9);
  v7 = *a1;
  v10 = v6;
  v11 = v7;
  if ( sub_13470(&v10, &v11) )
    return (v10 - *a1 - 2) >> 1;
  else
    return -1;
}


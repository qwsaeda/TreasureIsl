/* ea:   0x00030810 */
/* name: sub_30810 */
/* size: 196 */
unsigned int sub_30810(int *a1, ...)
{
  __int16 v2; // r6
  int v3; // r4
  unsigned __int16 *v4; // r4
  unsigned __int16 v5; // r3
  int v6; // r0
  unsigned int result; // r0
  int v8; // [sp+0h] [bp-34h] BYREF
  _BYTE v9[2]; // [sp+8h] [bp-2Ch] BYREF
  int v10; // [sp+Ah] [bp-2Ah]
  int varg_r1; // [sp+28h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+28h] [bp-Ch]
  int varg_r2; // [sp+2Ch] [bp-8h]
  va_list varg_r3; // [sp+30h] [bp-4h] BYREF

  va_start(varg_r3, a1);
  va_start(varg_r1a, a1);
  varg_r1 = va_arg(varg_r3, _DWORD);
  varg_r2 = va_arg(varg_r3, _DWORD);
  v2 = varg_r2;
  v3 = *a1;
  if ( sub_39798() == v3 )
  {
    v8 = 65470464;
    v9[0] = varg_r1;
    v10 = 65470464;
    sub_3565C(a1, &v8, v9);
  }
  v4 = (unsigned __int16 *)sub_308D4(a1, (int *)varg_r1a);
  v5 = v4[1];
  v6 = (unsigned __int16)(*v4 + v2);
  *v4 = v6;
  result = v6 << 16;
  if ( HIWORD(result) > v5 )
    *v4 = v5;
  return result;
}


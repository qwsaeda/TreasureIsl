/* ea:   0x0002F0E8 */
/* name: sub_2F0E8 */
/* size: 120 */
_DWORD *sub_2F0E8(_DWORD *result, ...)
{
  int v1; // r2
  _DWORD *v2; // r4
  _DWORD *v3; // r0
  int varg_r1; // [sp+14h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+14h] [bp-Ch]
  int varg_r2; // [sp+18h] [bp-8h]
  va_list varg_r3; // [sp+1Ch] [bp-4h] BYREF

  va_start(varg_r3, result);
  va_start(varg_r1a, result);
  varg_r1 = va_arg(varg_r3, _DWORD);
  varg_r2 = va_arg(varg_r3, _DWORD);
  v1 = varg_r2;
  v2 = result;
  if ( varg_r1 <= 4 )
  {
    result[178] = varg_r1;
    result[176] = 16 * v1;
    result[179] = 27;
    result[180] = 120;
    sub_3402C(result + 18);
    if ( varg_r1 == 1 )
    {
      return (_DWORD *)sub_51940(v2 + 72);
    }
    else
    {
      v3 = (_DWORD *)sub_1DD10(v2 + 124, (int *)varg_r1a);
      return (_DWORD *)sub_51940(*v3);
    }
  }
  return result;
}


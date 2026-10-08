/* ea:   0x0002F160 */
/* name: sub_2F160 */
/* size: 132 */
int sub_2F160(int a1, int a2, ...)
{
  int result; // r0
  int varg_r2; // [sp+20h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+20h] [bp-8h]
  va_list varg_r3; // [sp+24h] [bp-4h] BYREF

  va_start(varg_r3, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(varg_r3, _DWORD);
  if ( a2 == varg_r2 )
  {
    *(_DWORD *)(a1 + 120) = 0;
    *(_DWORD *)(a1 + 124) = 0;
    sub_2F708(a1 + 100);
    sub_2F708(a1 + 88);
    *(_DWORD *)(a1 + 704) = 0;
    *(_DWORD *)(a1 + 712) = a2;
    *(_DWORD *)(a1 + 716) = 27;
    *(_DWORD *)(a1 + 720) = 120;
    return 27;
  }
  else if ( varg_r2 == 1 )
  {
    *(_BYTE *)(a1 + 492) = 1;
    return 1;
  }
  else
  {
    result = (int)sub_1EDAC(a1 + 496, (int *)varg_r2a);
    *(_BYTE *)(*(_DWORD *)(result + 20) + 204) = 1;
  }
  return result;
}


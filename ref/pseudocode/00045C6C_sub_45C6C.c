/* ea:   0x00045C6C */
/* name: sub_45C6C */
/* size: 272 */
int __fastcall sub_45C6C(int a1, int a2, int a3, int a4, unsigned int a5, _DWORD *a6, int a7)
{
  int result; // r0
  bool v9; // zf

  if ( a7 != 0 )
    return 0;
  switch ( a3 )
  {
    case 275:
      SystemIdleTimerReset();
      SHIdleTimerReset();
      *a6 = 0;
      return 1;
    case 8:
      *(_DWORD *)(a1 + 108) = 1;
      sub_45D7C(a1 + 36);
      KillTimer(*(HWND *)(a1 + 4), 0x65u);
      *a6 = 0;
      return 1;
    case 7:
      *(_DWORD *)(a1 + 108) = 0;
      sub_45DB8(a1 + 36);
      SetTimer(*(HWND *)(a1 + 4), 0x65u, 0x1388u, nullptr);
      *a6 = 0;
      return 1;
    default:
      break;
  }
  v9 = sub_45A38(a1, a2, a3, a4, a5, a6, 0) == 0;
  result = 1;
  if ( v9 )
    return 0;
  return result;
}


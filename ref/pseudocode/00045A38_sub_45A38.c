/* ea:   0x00045A38 */
/* name: sub_45A38 */
/* size: 520 */
int __fastcall sub_45A38(int a1, int a2, int a3, int a4, unsigned int a5, _DWORD *a6, int a7)
{
  if ( a7 != 0 )
    return 0;
  switch ( a3 )
  {
    case 256:
      (*(void (__fastcall **)(int, _DWORD, int, unsigned int))(*(_DWORD *)(a1 - 44) + 28))(a1 - 44, 0, a4, a5);
      *a6 = 0;
      return 1;
    case 257:
      (*(void (__fastcall **)(int, int, int, unsigned int))(*(_DWORD *)(a1 - 44) + 28))(a1 - 44, 1, a4, a5);
      *a6 = 0;
      return 1;
    case 512:
      (*(void (__fastcall **)(int, _DWORD, _DWORD, unsigned int, int))(*(_DWORD *)(a1 - 44) + 32))(
        a1 - 44,
        0,
        (unsigned __int16)a5,
        HIWORD(a5),
        a4);
      *a6 = 0;
      return 1;
    case 513:
      (*(void (__fastcall **)(int, int, _DWORD, unsigned int, int))(*(_DWORD *)(a1 - 44) + 32))(
        a1 - 44,
        1,
        (unsigned __int16)a5,
        HIWORD(a5),
        a4);
      *a6 = 0;
      return 1;
    case 514:
      (*(void (__fastcall **)(int, int, _DWORD, unsigned int, int))(*(_DWORD *)(a1 - 44) + 32))(
        a1 - 44,
        2,
        (unsigned __int16)a5,
        HIWORD(a5),
        a4);
      *a6 = 0;
      return 1;
    default:
      break;
  }
  if ( a3 != 16 )
    return 0;
  (*(void (__fastcall **)(int))(*(_DWORD *)(a1 - 44) + 8))(a1 - 44);
  *a6 = 0;
  return 1;
}


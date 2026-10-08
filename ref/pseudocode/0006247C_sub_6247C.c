/* ea:   0x0006247C */
/* name: sub_6247C */
/* size: 200 */
int __fastcall sub_6247C(
        int a1,
        _DWORD *a2,
        int a3,
        int a4,
        int (__fastcall *a5)(int, _DWORD *, int, int, int),
        int a6)
{
  int result; // r0
  int v11; // r0

  if ( *(_BYTE *)(a1 + 4) == 0 )
    __debugbreak();
  if ( *(_DWORD *)(a1 + 12) >= a2[1]
    || *(_DWORD *)(a1 + 16) <= *a2
    || *(_DWORD *)(a1 + 20) >= a2[3]
    || (LOBYTE(result) = 0, *(_DWORD *)(a1 + 24) <= a2[2]) )
  {
    LOBYTE(result) = 1;
  }
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v11 = (*(int (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 8) + 12))(*(_DWORD *)(a1 + 8));
    return a5(v11, a2, a3, a4, a6);
  }
  return result;
}


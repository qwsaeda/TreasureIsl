/* ea:   0x00063BE8 */
/* name: sub_63BE8 */
/* size: 160 */
int __fastcall sub_63BE8(int a1, int a2)
{
  if ( a2 == 0 )
    __debugbreak();
  if ( *(_BYTE *)(a1 + 4) != 0 )
    (*(void (__fastcall **)(int))(*(_DWORD *)a1 + 8))(a1);
  if ( (*(unsigned __int8 (__fastcall **)(int, int, _DWORD, _DWORD))(*(_DWORD *)(a1 + 20) + 4))(a1 + 20, a2, 0, 0) == 0
    || (unsigned __int8)sub_63D88(a1) == 0
    || (unsigned __int8)sub_63DD4(a1) == 0 )
  {
    return 0;
  }
  *(_BYTE *)(a1 + 4) = 1;
  *(_DWORD *)(a1 + 44) = *(_DWORD *)(a1 + 12);
  return *(unsigned __int8 *)(a1 + 4);
}


/* ea:   0x00045DF4 */
/* name: sub_45DF4 */
/* size: 92 */
int __fastcall sub_45DF4(int a1)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 152);
  if ( result == 0 )
  {
    *(_DWORD *)(a1 + 144) = GetTickCount();
    if ( *(_BYTE *)(a1 + 92) != 0 )
      (*(void (__fastcall **)(int))(*(_DWORD *)a1 + 8))(a1);
    else
      sub_3D8D8((_DWORD *)(a1 + 80));
    return sub_45E50(a1 + 144);
  }
  return result;
}


/* ea:   0x0003DB48 */
/* name: sub_3DB48 */
/* size: 184 */
int __fastcall sub_3DB48(int a1, int a2, int a3)
{
  int result; // r0
  int v6; // r1
  char v7; // r0

  if ( a3 == 117 )
  {
    if ( a2 == 0 )
      return sub_3EAC4(*(_DWORD *)(a1 + 28));
  }
  else if ( a3 == 118 )
  {
    if ( a2 == 0 )
      return sub_3EA64(*(_DWORD *)(a1 + 28));
  }
  else if ( a2 == 0 && a3 == 13 )
  {
    v6 = *(_DWORD *)(a1 + 60);
    if ( v6 != 0 )
    {
      if ( *(_DWORD *)(v6 + 16) != 0 || (v7 = 0, *(_DWORD *)(v6 + 20) != 0) )
        v7 = 1;
      if ( v7 != 0 )
        *(_BYTE *)(a1 + 12) = 1;
    }
  }
  result = *(_DWORD *)(a1 + 8);
  if ( result != 0 )
    return (*(int (__fastcall **)(int, int))(*(_DWORD *)result + 16))(result, a2);
  return result;
}


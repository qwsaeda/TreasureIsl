/* ea:   0x0004CB90 */
/* name: sub_4CB90 */
/* size: 224 */
int __fastcall sub_4CB90(_DWORD *a1)
{
  int result; // r0
  int v3; // r2
  int v4; // r1
  int v5; // r4
  unsigned int v6; // r2
  unsigned int v7; // r1

  result = a1[5];
  v3 = result;
  if ( result != 0 )
  {
    v4 = *(unsigned __int8 *)(result + 50);
    result = a1[2];
    if ( v4 == result )
    {
      result = *(unsigned __int16 *)(v3 + 22);
      if ( *(_WORD *)(v3 + 22) == 0 )
      {
        (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(a1[3] + 40) + 32))(
          *(_DWORD *)(a1[3] + 40),
          *(_DWORD *)(*(_DWORD *)(a1[6] + 32) + 8));
        v5 = a1[6] + 144;
        v6 = *(_DWORD *)(a1[6] + 152) - 1;
        *(_DWORD *)(a1[6] + 152) = v6;
        if ( v6 / 7 < v6 / 0xA )
          *(_DWORD *)(v5 + 28) = 0;
        v7 = *(unsigned __int8 *)(a1[5] + 48);
        if ( v7 < 0xA )
        {
          --*(_BYTE *)(v7 + a1[47]);
          --a1[50];
        }
        result = a1[5];
        *(_BYTE *)(result + 50) = 0;
      }
    }
  }
  return result;
}


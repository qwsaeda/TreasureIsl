/* ea:   0x00048A50 */
/* name: sub_48A50 */
/* size: 112 */
int __fastcall sub_48A50(int result, _DWORD *a2, char a3)
{
  unsigned int *v4; // r5
  unsigned int v5; // r3
  unsigned int v6; // r2
  unsigned int v7; // r1
  unsigned int v8; // r1

  if ( a2 != nullptr )
  {
    if ( a3 != 0 )
    {
      v4 = (unsigned int *)(result + 144);
      v5 = (unsigned int)(a2[2] + 1) >> 1;
      v6 = *(_DWORD *)(result + 148);
      v7 = v5 + *(_DWORD *)(result + 144);
      *(_DWORD *)(result + 144) = v7;
      result = 0;
      if ( v7 > v6 )
        *v4 = v6;
      v8 = a2[2];
      if ( v5 <= v8 )
        result = v8 - v5;
      a2[2] = result;
    }
    else
    {
      return sub_517EC(result, 3 * a2[1], a2[5]);
    }
  }
  return result;
}


/* ea:   0x0001B150 */
/* name: sub_1B150 */
/* size: 296 */
unsigned int __fastcall sub_1B150(unsigned int result, int a2, int a3, int a4, __int16 a5, int a6)
{
  unsigned __int16 *v9; // r6
  int i; // r11
  int v11; // r5
  unsigned __int16 v12; // r4
  int v13; // r0
  int v14; // r3

  if ( a6 != -1 )
  {
    v9 = (unsigned __int16 *)((*(int (__fastcall **)(_DWORD))(**(_DWORD **)result + 12))(*(_DWORD *)result)
                            + 2 * ((24 * a6) >> 1));
    for ( i = 12; i != 0; --i )
    {
      v11 = 0;
      v12 = *v9;
      do
      {
        result = (v12 & (1 << (v11 & 0x1F))) != 0;
        if ( (v12 & (1 << (v11 & 0x1F))) != 0 )
        {
          v13 = (*(int (__fastcall **)(int))(*(_DWORD *)a2 + 40))(a2);
          v14 = *(_DWORD *)(v13 + 28);
          result = ((unsigned int)(*(_DWORD *)(v13 + 24) * a4) >> 1) - v11;
          if ( v11 >= 8 )
            *(_WORD *)(v14 + 2 * (result + a3) + 48) = a5;
          else
            *(_WORD *)(v14 + 2 * (result + a3) + 16) = a5;
        }
        ++v11;
      }
      while ( v11 < 16 );
      ++v9;
      ++a4;
    }
  }
  return result;
}


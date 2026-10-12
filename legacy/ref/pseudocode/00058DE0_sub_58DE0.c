/* ea:   0x00058DE0 */
/* name: sub_58DE0 */
/* size: 476 */
int __fastcall sub_58DE0(int a1, int a2)
{
  __int64 *v4; // r4
  __int64 *v5; // r5
  int v6; // r2
  __int64 *v7; // r4
  unsigned int v8; // r3
  __int16 v9; // r1
  int v10; // r0
  int v12; // r0
  unsigned int v13; // r0
  unsigned int v14; // [sp+0h] [bp-24h] BYREF
  __int64 i; // [sp+8h] [bp-1Ch]

  sub_58DAC(a1);
  if ( a2 != 0 )
  {
    v4 = (__int64 *)(a2 + 12);
    i = *v4;
    if ( (_DWORD)i == 544501094 )
    {
      v5 = v4 + 1;
      v6 = *((unsigned __int16 *)v4 + 11);
      if ( (v6 == 4 || v6 == 8 || v6 == 16) && *((_WORD *)v4 + 5) == 1 )
      {
        v7 = (__int64 *)((char *)v5 + HIDWORD(i));
        for ( i = *(__int64 *)((char *)v5 + HIDWORD(i)); (_DWORD)i != 1635017060; i = *v7 )
          v7 = (__int64 *)((char *)v7 + HIDWORD(i) + 8);
        v8 = HIDWORD(i);
        *(_DWORD *)(a1 + 12) = *((_DWORD *)v5 + 1);
        *(_DWORD *)(a1 + 4) = v7 + 1;
        v9 = *((_WORD *)v5 + 7);
        v14 = v8;
        if ( v9 != 4 )
        {
          if ( v9 == 8 )
          {
            *(_DWORD *)(a1 + 16) = v8;
            *(_BYTE *)(a1 + 112) = 0;
            v12 = *(_DWORD *)(a1 + 12);
            *(_BYTE *)(a1 + 8) = 1;
            *(_DWORD *)(a1 + 104) = v12;
            return 1;
          }
          if ( v9 == 16 )
          {
            *(_DWORD *)(a1 + 16) = v8 >> 1;
            v10 = *(_DWORD *)(a1 + 12);
            *(_BYTE *)(a1 + 8) = 0;
            *(_DWORD *)(a1 + 104) = v10;
            *(_BYTE *)(a1 + 112) = 0;
            return 1;
          }
          goto LABEL_19;
        }
        if ( *(_WORD *)v5 == 2
          && sub_58808(a1, (unsigned __int8 *)v5) >= 0
          && sub_58A0C(a1, (void **)(a1 + 4), (int *)&v14, 0) >= 0 )
        {
          v13 = v14;
          *(_BYTE *)(a1 + 112) = 1;
          *(_DWORD *)(a1 + 16) = v13 >> 1;
          *(_BYTE *)(a1 + 8) = 0;
LABEL_19:
          *(_DWORD *)(a1 + 104) = *(_DWORD *)(a1 + 12);
          return 1;
        }
      }
    }
  }
  return 0;
}


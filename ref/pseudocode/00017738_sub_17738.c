/* ea:   0x00017738 */
/* name: sub_17738 */
/* size: 344 */
int __fastcall sub_17738(int *a1, int a2, unsigned __int16 *a3)
{
  int v6; // r3
  int v7; // r6
  int v8; // r5
  bool v9; // r0
  unsigned int v10; // r2
  int v11; // r4
  int v12; // r1
  int v14; // r1
  int v15; // [sp+8h] [bp-34h] BYREF
  int v16; // [sp+Ch] [bp-30h] BYREF
  int v17; // [sp+10h] [bp-2Ch] BYREF
  char v18; // [sp+14h] [bp-28h]

  v6 = *a1;
  v7 = *a1;
  v8 = *(_DWORD *)(*a1 + 4);
  v9 = true;
  if ( v8 != 0 )
  {
    v10 = *a3;
    do
    {
      v7 = v8;
      v9 = v10 < *(unsigned __int16 *)(v8 + 16);
      if ( v10 >= *(unsigned __int16 *)(v8 + 16) )
        v8 = *(_DWORD *)(v8 + 12);
      else
        v8 = *(_DWORD *)(v8 + 8);
    }
    while ( v8 != 0 );
  }
  v11 = v7;
  if ( v9 )
  {
    if ( v7 == *(_DWORD *)(v6 + 8) )
    {
      v12 = *sub_17644(a1, &v15, v8, v7, a3, 0);
      *(_BYTE *)(a2 + 4) = 1;
      *(_DWORD *)a2 = v12;
      return a2;
    }
    v11 = sub_17924(v7);
  }
  if ( *(unsigned __int16 *)(v11 + 16) >= (unsigned int)*a3 )
  {
    v17 = v11;
    v18 = 0;
    sub_18350(a2, &v17);
  }
  else
  {
    v14 = *sub_17644(a1, &v16, v8, v7, a3, 0);
    *(_BYTE *)(a2 + 4) = 1;
    *(_DWORD *)a2 = v14;
  }
  return a2;
}


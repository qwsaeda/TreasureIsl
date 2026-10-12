/* ea:   0x00011248 */
/* name: sub_11248 */
/* size: 536 */
int __fastcall sub_11248(int a1, int a2, _DWORD *a3, int a4, int a5, int a6)
{
  char v9; // r1
  unsigned int v10; // r0
  int v11; // r5
  int v12; // r3
  int v13; // r0
  int v14; // r0
  int v15; // r0
  _BYTE v17[8]; // [sp+0h] [bp-38h] BYREF
  int v18; // [sp+8h] [bp-30h] BYREF
  unsigned int v19; // [sp+Ch] [bp-2Ch]
  int v20; // [sp+10h] [bp-28h]
  int v21; // [sp+14h] [bp-24h]
  int v22; // [sp+18h] [bp-20h]
  int v23; // [sp+1Ch] [bp-1Ch]

  *(_DWORD *)(a1 + 4) = a2;
  *(_DWORD *)(a1 + 8) = *(_DWORD *)a4;
  *(_DWORD *)(a1 + 12) = *(_DWORD *)(a4 + 4);
  *(_DWORD *)(a1 + 16) = *(_DWORD *)(a4 + 8);
  *(_BYTE *)(a1 + 22) = *(_BYTE *)(a4 + 14);
  *(_BYTE *)(a1 + 23) = *(_BYTE *)(a4 + 15);
  *(_WORD *)(a1 + 20) = *(_WORD *)(a4 + 12);
  *(_WORD *)(a1 + 24) = *(_WORD *)(a4 + 18);
  *(_DWORD *)(a1 + 28) = *(_DWORD *)(a4 + 32);
  v9 = *(_BYTE *)(a1 + 22);
  *(_DWORD *)(a1 + 32) = a5;
  if ( (v9 & 3) != 1 )
    goto LABEL_20;
  v10 = v9 & 0x3F;
  v18 = 0;
  v19 = 0;
  v20 = 0;
  v21 = 0;
  v22 = 0;
  v23 = 0;
  if ( v10 <= 0x15 )
  {
    if ( (v9 & 0x3F) == 0x15 )
    {
      v11 = *(_DWORD *)(a3[8] + 132);
      goto LABEL_14;
    }
    if ( v10 == 5 )
    {
      v11 = *(_DWORD *)(a3[8] + 4 * *(_DWORD *)(a3[66] + 4 * sub_53940(a3 + 13, a1) - 4) + 148);
      goto LABEL_14;
    }
    if ( v10 == 17 )
    {
      v11 = *(_DWORD *)(a3[8] + 136);
      goto LABEL_14;
    }
    goto LABEL_11;
  }
  if ( v10 == 25 )
  {
    v11 = *(_DWORD *)(a3[8] + 140);
  }
  else
  {
    if ( v10 != 29 )
    {
LABEL_11:
      v11 = *(_DWORD *)(a3[8] + 8);
      goto LABEL_14;
    }
    v11 = *(_DWORD *)(a3[8] + 144);
  }
LABEL_14:
  if ( v11 != 0 )
  {
    v12 = *(_DWORD *)(a1 + 16);
    v19 = *(_DWORD *)(a1 + 12) + ((unsigned int)(64 - *(_DWORD *)(v11 + 4)) >> 1);
    v20 = v12 + ((unsigned int)(32 - *(_DWORD *)(v11 + 8)) >> 1);
    if ( operator new(0x14u) != nullptr )
      v13 = sub_430AC();
    else
      v13 = 0;
    *(_DWORD *)(a1 + 40) = v13;
    if ( v13 == 0 )
      return 0;
    sub_11558(a3 + 17, v17, a3[17], a1 + 40);
    v14 = *(_DWORD *)(*(_DWORD *)(a3[17] + 4) + 8);
    (*(void (__fastcall **)(int, int, int *, int))(*(_DWORD *)v14 + 8))(v14, v11, &v18, 1);
  }
LABEL_20:
  if ( (*(_BYTE *)(a4 + 14) & 0x3F) == 1 )
  {
    if ( operator new(0x34u) != nullptr )
      v15 = sub_44AFC();
    else
      v15 = 0;
    *(_DWORD *)(a1 + 36) = v15;
    if ( v15 == 0 )
      return 0;
    sub_44B80(v15, a4, a6, a1);
  }
  return 1;
}


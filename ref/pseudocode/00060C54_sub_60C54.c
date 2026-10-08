/* ea:   0x00060C54 */
/* name: sub_60C54 */
/* size: 320 */
int __fastcall sub_60C54(char **a1, int a2, _DWORD *a3)
{
  char *v6; // r2
  char *v7; // r5
  char *v8; // r4
  bool v9; // r0
  int v10; // r3
  int v11; // r1
  int v13; // r1
  _BYTE v14[4]; // [sp+8h] [bp-30h] BYREF
  _BYTE v15[4]; // [sp+Ch] [bp-2Ch] BYREF
  int v16; // [sp+10h] [bp-28h] BYREF
  char v17; // [sp+14h] [bp-24h]

  v6 = *a1;
  v7 = *a1;
  v8 = *((char **)*a1 + 1);
  v9 = true;
  while ( v8 != nullptr )
  {
    v7 = v8;
    v9 = *a3 < *((_DWORD *)v8 + 4);
    if ( *a3 >= *((_DWORD *)v8 + 4) )
      v8 = *((char **)v8 + 3);
    else
      v8 = *((char **)v8 + 2);
  }
  v10 = (int)v7;
  if ( v9 )
  {
    if ( v7 == *((char **)v6 + 2) )
    {
      v11 = *(_DWORD *)sub_61680(a1, v14, v8, v7, a3, 0);
      *(_BYTE *)(a2 + 4) = 1;
      *(_DWORD *)a2 = v11;
      return a2;
    }
    v10 = sub_17924(v7);
  }
  if ( *(_DWORD *)(v10 + 16) >= *a3 )
  {
    v16 = v10;
    v17 = 0;
    sub_603B4(a2, (int)&v16);
  }
  else
  {
    v13 = *(_DWORD *)sub_61680(a1, v15, v8, v7, a3, 0);
    *(_BYTE *)(a2 + 4) = 1;
    *(_DWORD *)a2 = v13;
  }
  return a2;
}


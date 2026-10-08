/* ea:   0x00051940 */
/* name: sub_51940 */
/* size: 724 */
int __fastcall sub_51940(int a1, int a2)
{
  int v4; // r1
  int v5; // r2
  char v6; // r10
  int *v7; // r11
  char v8; // r8
  unsigned __int16 v9; // r7
  unsigned __int16 v10; // r6
  int result; // r0
  char v12; // r4
  int v13; // r3
  char v14; // r5
  unsigned __int8 v15; // r2
  unsigned __int8 v16; // r5
  unsigned int v17; // r4
  unsigned __int8 v18; // r0
  char v19; // r0
  int v20; // r0
  int v21; // r2
  int v22; // [sp+0h] [bp-34h] BYREF
  int v23; // [sp+4h] [bp-30h] BYREF
  int v24; // [sp+8h] [bp-2Ch]
  char v25; // [sp+Ch] [bp-28h] BYREF

  v24 = a2;
  v4 = *(_DWORD *)(a1 + 64);
  v5 = *(_DWORD *)(a1 + 68);
  v6 = 0;
  v7 = (int *)(a1 + 184);
  v8 = *(_BYTE *)(a1 + 128);
  v22 = *(_DWORD *)(a1 + 12);
  v9 = v4;
  v10 = v5;
  result = sub_52CE8(a1 + 184);
  if ( a2 > 0 )
  {
    do
    {
      v12 = *(_BYTE *)(a1 + 129);
      v13 = v22 + 20;
      v14 = *(_BYTE *)(a1 + 130);
      v15 = (v8 & *(_BYTE *)(v22 + 23) & 0xF | v12) & v14;
      if ( v15 == 0 )
      {
        *(_BYTE *)(a1 + 128) = 15;
        v8 = 15;
        v15 = (*(_BYTE *)(v13 + 3) & 0xF | v12) & v14;
      }
      *(_BYTE *)(a1 + 129) = 0;
      *(_BYTE *)(a1 + 130) = -1;
      switch ( v15 )
      {
        case 1u:
          --v10;
          v8 = 11;
          break;
        case 2u:
          ++v9;
          v8 = 7;
          break;
        case 4u:
          ++v10;
          v8 = 14;
          break;
        case 8u:
          --v9;
          v8 = 13;
          break;
        default:
          v6 = 1;
          break;
      }
      if ( v6 != 0 )
      {
        v23 = v15;
        v16 = rand() % 4;
        v17 = v16;
        if ( (unsigned __int8)sub_52998(&v23, v16) == 0 )
        {
          do
          {
            if ( v17 < 3 )
              v18 = v17 + 1;
            else
              v18 = 0;
            v16 = v18;
            v17 = v18;
          }
          while ( (unsigned __int8)sub_52998(&v23, v18) == 0 );
        }
        switch ( v16 )
        {
          case 0u:
            --v10;
            v19 = 11;
            goto LABEL_21;
          case 1u:
            ++v9;
            v8 = 7;
            goto LABEL_22;
          case 2u:
            ++v10;
            v8 = 14;
            goto LABEL_22;
          case 3u:
            --v9;
            v19 = 13;
LABEL_21:
            v8 = v19;
LABEL_22:
            v6 = 0;
            break;
          default:
            break;
        }
      }
      v20 = sub_53808(*(_DWORD *)(a1 + 24) + 52, v9, v10);
      v21 = *v7;
      v22 = v20;
      result = sub_52A28(v7, &v25, v21, &v22);
      --v24;
    }
    while ( v24 != 0 );
  }
  *(_BYTE *)(a1 + 128) = v8;
  return result;
}


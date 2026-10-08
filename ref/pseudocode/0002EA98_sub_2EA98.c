/* ea:   0x0002EA98 */
/* name: sub_2EA98 */
/* size: 924 */
int __fastcall sub_2EA98(_DWORD *a1, const WCHAR *a2)
{
  _DWORD *v4; // r8
  HANDLE FileW; // r11
  int v6; // r7
  int v7; // r6
  int v8; // r0
  int v9; // r0
  int v10; // r4
  int v11; // r6
  int *v13; // r1
  int v14; // r0
  DWORD NumberOfBytesRead; // [sp+Ch] [bp-60h] BYREF
  int v16; // [sp+10h] [bp-5Ch] BYREF
  int v17; // [sp+14h] [bp-58h] BYREF
  int v18; // [sp+18h] [bp-54h] BYREF
  int v19; // [sp+1Ch] [bp-50h] BYREF
  _DWORD Buffer[4]; // [sp+24h] [bp-48h] BYREF
  unsigned __int16 v21[8]; // [sp+34h] [bp-38h] BYREF
  int v22; // [sp+44h] [bp-28h]

  v4 = a1 + 13;
  sub_536F4(a1 + 13);
  sub_36E60(a1 + 16);
  a1[173] = 0;
  a1[174] = 0;
  v16 = 0;
  FileW = CreateFileW(a2, 0x80000000, 1u, nullptr, 3u, 0x80u, nullptr);
  ReadFile(FileW, a1 + 173, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(FileW, a1 + 174, 4u, &NumberOfBytesRead, nullptr);
  sub_53658(v4, a1[173], a1[174]);
  v6 = a1[174] - 1;
  if ( v6 >= 0 )
  {
    while ( 1 )
    {
      v7 = 0;
      if ( (int)a1[173] > 0 )
        break;
LABEL_20:
      if ( --v6 < 0 )
        goto LABEL_21;
    }
    while ( 1 )
    {
      ReadFile(FileW, Buffer, 0x24u, &NumberOfBytesRead, nullptr);
      if ( Buffer[0] < 0 )
        goto LABEL_19;
      switch ( v22 )
      {
        case 1:
          if ( operator new(0x2Cu) == nullptr )
            goto LABEL_13;
          v8 = sub_43214();
          break;
        case 3:
          if ( operator new(0x30u) == nullptr )
            goto LABEL_13;
          v8 = sub_4F584();
          break;
        case 6:
          if ( operator new(0x30u) == nullptr )
          {
LABEL_13:
            v8 = 0;
            break;
          }
          v8 = sub_4E3A4();
          break;
        default:
          goto LABEL_15;
      }
      sub_537F8(v4, v6, v7, v8);
LABEL_15:
      if ( sub_53808(v4, v6, v7) == 0 )
        return 0;
      v9 = sub_53808(v4, v6, v7);
      if ( (unsigned __int8)sub_11248(
                              v9,
                              *(_DWORD *)(a1[5] + 4 * Buffer[0]),
                              a1,
                              (int)Buffer,
                              *(_DWORD *)(a1[11] + 4 * Buffer[0]),
                              a1[173]) == 0 )
        return 0;
      if ( *(_DWORD *)(sub_53808(v4, v6, v7) + 36) != 0 )
      {
        v10 = sub_53808(v4, v6, v7);
        *(_DWORD *)sub_1623C(a1 + 127, v21) = *(_DWORD *)(v10 + 36);
      }
LABEL_19:
      if ( ++v7 >= a1[173] )
        goto LABEL_20;
    }
  }
LABEL_21:
  ReadFile(FileW, &v16, 4u, &NumberOfBytesRead, nullptr);
  v11 = 0;
  if ( v16 > 0 )
  {
    while ( 1 )
    {
      ReadFile(FileW, Buffer, 0x24u, &NumberOfBytesRead, nullptr);
      if ( v22 == 2 )
      {
        if ( operator new(0x14u) != nullptr )
          v19 = sub_430AC();
        else
          v19 = 0;
        v13 = &v19;
        goto LABEL_39;
      }
      if ( v22 == 4 )
        break;
      if ( v22 == 5 )
      {
        if ( operator new(0x18u) != nullptr )
          v17 = sub_4E14C();
        else
          v17 = 0;
        v13 = &v17;
LABEL_39:
        sub_2F770(a1 + 16, v13);
      }
      v14 = *(_DWORD *)(a1[16] + 4);
      if ( *(_DWORD *)(v14 + 8) == 0 )
        return 0;
      (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD *, _DWORD))(**(_DWORD **)(v14 + 8) + 12))(
        *(_DWORD *)(v14 + 8),
        *(_DWORD *)(a1[5] + 4 * Buffer[0]),
        Buffer,
        *(_DWORD *)(a1[11] + 4 * Buffer[0]));
      if ( ++v11 >= v16 )
        goto LABEL_42;
    }
    if ( operator new(0x1Cu) != nullptr )
      v18 = sub_4F388();
    else
      v18 = 0;
    v13 = &v18;
    goto LABEL_39;
  }
LABEL_42:
  CloseHandle(FileW);
  sub_3958C(a1 + 16, sub_34FF8);
  return 1;
}


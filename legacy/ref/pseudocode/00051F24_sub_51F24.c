/* ea:   0x00051F24 */
/* name: sub_51F24 */
/* size: 348 */
int __fastcall sub_51F24(int a1)
{
  int result; // r0
  int v3; // r6
  int v4; // r5
  int v5; // r0
  int v6; // r0
  int v7; // r1
  int v8; // r0
  int v9; // r0

  result = *(_DWORD *)(a1 + 8);
  if ( result <= 4 )
  {
    *(_DWORD *)(a1 + 120) = 0;
    sub_52CE8(a1 + 184);
    v3 = sub_53890(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 8));
    v4 = *(_DWORD *)(a1 + 24);
    *(_DWORD *)(a1 + 12) = v3;
    *(_DWORD *)(a1 + 64) = (int)*(unsigned __int16 *)(v3 + 20) % *(_DWORD *)(v4 + 692);
    *(_DWORD *)(a1 + 68) = *(unsigned __int16 *)(v3 + 20) / *(int *)(v4 + 692);
    v5 = *(_DWORD *)(v3 + 36);
    *(_DWORD *)(a1 + 124) = 3;
    *(_DWORD *)(a1 + 20) = v5;
    *(_BYTE *)(a1 + 130) = -1;
    *(_BYTE *)(a1 + 129) = 0;
    *(_BYTE *)(a1 + 128) = 15;
    *(_DWORD *)sub_2FA58((_DWORD *)(v4 + 532), (int *)(a1 + 8)) = 56 - *(_DWORD *)(v3 + 12);
    *(_DWORD *)(sub_2FA58((_DWORD *)(*(_DWORD *)(a1 + 24) + 532), (int *)(a1 + 8)) + 4) = 94
                                                                                        - *(_DWORD *)(*(_DWORD *)(a1 + 12) + 16);
    v6 = *(_DWORD *)(a1 + 48);
    *(_DWORD *)(a1 + 40) = v6;
    if ( v6 != 0 )
    {
      v7 = *(_DWORD *)(a1 + 100);
      *(_DWORD *)(a1 + 40) = *(_DWORD *)(a1 + 44);
      v8 = *(_DWORD *)(a1 + 80);
      *(_DWORD *)(a1 + 96) = v7;
      *(_DWORD *)(a1 + 72) = v8;
      v9 = *(_DWORD *)(a1 + 32);
      *(_DWORD *)(a1 + 76) = *(_DWORD *)(a1 + 84);
      *(_DWORD *)(a1 + 108) = 0;
      *(_DWORD *)(a1 + 28) = v9;
    }
    sub_3958C((_DWORD **)(*(_DWORD *)(a1 + 24) + 64));
    return sub_2F160(*(_DWORD *)(a1 + 24), *(_DWORD *)(a1 + 8), *(_DWORD *)(a1 + 8));
  }
  return result;
}


/* ea:   0x00041354 */
/* name: sub_41354 */
/* size: 2056 */
int __fastcall sub_41354(int result, int a2, int a3)
{
  int *v3; // r7
  int v5; // r4
  void (__fastcall ***v6)(int, int); // r3
  int v7; // r0
  int (__fastcall ***v8)(int, int); // r3
  int v9; // r0
  int v10; // r0
  int v11; // r0
  int v12; // r0
  void *v13; // r0
  void *v14; // r0
  _DWORD *v15; // r4
  _DWORD *v16; // r0
  int v17; // r0
  int v18; // r0
  void *v19; // r0
  void *v20; // r0
  _DWORD *v21; // r4
  _DWORD *v22; // r0
  int v23; // r0
  __int16 **v24; // r0
  __int16 **v25; // r0
  wchar_t *v26; // r0
  wchar_t *v27; // r3
  _DWORD *v28; // r4
  _DWORD *v29; // r0
  _DWORD *v30; // r0
  _DWORD *v31; // r0
  wchar_t *v32; // r0
  wchar_t *v33; // r3
  _DWORD *v34; // r4
  _DWORD *v35; // r0
  _DWORD v36[4]; // [sp+14h] [bp-154h] BYREF
  _DWORD v37[4]; // [sp+24h] [bp-144h] BYREF
  _DWORD v38[4]; // [sp+34h] [bp-134h] BYREF
  void *v39[4]; // [sp+44h] [bp-124h] BYREF
  _DWORD v40[4]; // [sp+54h] [bp-114h] BYREF
  void *v41[4]; // [sp+64h] [bp-104h] BYREF
  _DWORD v42[4]; // [sp+74h] [bp-F4h] BYREF
  int v43[4]; // [sp+84h] [bp-E4h] BYREF
  int v44[4]; // [sp+94h] [bp-D4h] BYREF
  int v45[4]; // [sp+A4h] [bp-C4h] BYREF
  int v46[4]; // [sp+B4h] [bp-B4h] BYREF
  int v47[4]; // [sp+C4h] [bp-A4h] BYREF
  int v48[4]; // [sp+D4h] [bp-94h] BYREF
  int v49[4]; // [sp+E4h] [bp-84h] BYREF
  int v50[4]; // [sp+F4h] [bp-74h] BYREF
  _BYTE v51[100]; // [sp+104h] [bp-64h] BYREF

  v3 = (int *)result;
  if ( a2 == 0 )
  {
    if ( *(_DWORD *)(result + 52) != 0 )
    {
      (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(result + 52) + 16))(*(_DWORD *)(result + 52), 0);
      result = *(_DWORD *)(v3[13] + 1008);
      if ( result != 0 )
      {
        if ( off_8119C != nullptr )
        {
          v5 = *((_DWORD *)off_8119C + 257);
          off_8119C = nullptr;
          (*(void (__fastcall **)(int))(*(_DWORD *)v3[13] + 8))(v3[13]);
          v6 = (void (__fastcall ***)(int, int))v3[13];
          if ( v6 != nullptr )
            (**v6)(v3[13], 1);
          v7 = v3[1];
          v3[13] = 0;
          if ( v5 >= 5 )
            return sub_17C7C(v7, 0);
          else
            return sub_3DCE4(v7, v5);
        }
        else
        {
          if ( off_81198 != nullptr )
          {
            sub_30FDC(v3[1]);
            off_81198 = nullptr;
          }
          sub_17C7C(v3[1], 0);
          result = (*(int (__fastcall **)(int))(*(_DWORD *)v3[13] + 8))(v3[13]);
          v8 = (int (__fastcall ***)(int, int))v3[13];
          if ( v8 != nullptr )
            result = (**v8)(v3[13], 1);
          v3[13] = 0;
        }
      }
    }
    else
    {
      sub_1333C(v38);
      sub_1333C(v37);
      sub_1333C(v36);
      if ( a3 == 13 )
      {
        sub_17C7C(v3[1], 0);
        switch ( v3[14] )
        {
          case 0:
            sub_3DC04(v3[1], 8);
            sub_12EA0((int)v36);
            sub_12EA0((int)v37);
            return sub_12EA0((int)v38);
          case 1:
            (*(void (__fastcall **)(int *, _BYTE *, int))(*v3 + 24))(v3, v51, 5);
            sub_1309C((int)v37);
            v11 = sub_12F04(aDataUiPak);
            sub_134A4((int)v37, (int)aDataUiPak, (int)&aDataUiPak[v11]);
            sub_1309C((int)v36);
            v12 = sub_12F04(aDataSaveloadMa);
            sub_134A4((int)v36, (int)aDataSaveloadMa, (int)&aDataSaveloadMa[v12]);
            v13 = operator new(0x460u);
            if ( v13 != nullptr )
            {
              v14 = (void *)sub_4E60C(v13, v3[2]);
              off_8119C = v14;
              if ( v14 == nullptr )
                goto LABEL_49;
              sub_43D54(v14, *(_DWORD *)(v3[1] + 16));
              v15 = sub_132FC(v43, v36);
              v16 = sub_132FC(v48, v37);
              (*(void (__fastcall **)(void *, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)off_8119C + 4))(
                off_8119C,
                *v16,
                v16[1],
                v16[2],
                *v15,
                v15[1],
                v15[2]);
              sub_4EC40(off_8119C, 0, v51, v3[17]);
              v3[13] = (int)off_8119C;
              sub_12EA0((int)v36);
              sub_12EA0((int)v37);
              return sub_12EA0((int)v38);
            }
            else
            {
              off_8119C = nullptr;
              sub_12EA0((int)v36);
              sub_12EA0((int)v37);
              return sub_12EA0((int)v38);
            }
          case 2:
            sub_1309C((int)v37);
            v17 = sub_12F04(aDataUiPak);
            sub_134A4((int)v37, (int)aDataUiPak, (int)&aDataUiPak[v17]);
            sub_1309C((int)v36);
            v18 = sub_12F04(aDataOptionMap);
            sub_134A4((int)v36, (int)aDataOptionMap, (int)&aDataOptionMap[v18]);
            v19 = operator new(0x40Cu);
            if ( v19 != nullptr )
            {
              v20 = (void *)sub_4B0B0(v19, v3[2]);
              off_81198 = v20;
              if ( v20 == nullptr )
                goto LABEL_49;
              sub_43D54(v20, *(_DWORD *)(v3[1] + 16));
              v21 = sub_132FC(v46, v36);
              v22 = sub_132FC(v50, v37);
              (*(void (__fastcall **)(void *, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)off_81198 + 4))(
                off_81198,
                *v22,
                v22[1],
                v22[2],
                *v21,
                v21[1],
                v21[2]);
              sub_4B3CC(off_81198, *(unsigned __int8 *)(v3[1] + 32), *(unsigned __int8 *)(v3[1] + 33), v3[1]);
              v3[13] = (int)off_81198;
              sub_12EA0((int)v36);
              sub_12EA0((int)v37);
              result = sub_12EA0((int)v38);
            }
            else
            {
              off_81198 = nullptr;
              sub_12EA0((int)v36);
              sub_12EA0((int)v37);
              result = sub_12EA0((int)v38);
            }
            break;
          case 3:
            sub_1309C((int)v38);
            v23 = sub_12F04(aData);
            sub_134A4((int)v38, (int)aData, (int)&aData[v23]);
            v24 = (__int16 **)sub_37EDC(v41, (int)v38, (char *)aUiPak);
            if ( v24 != v37 )
              sub_17890((int)v37, *v24, v24[1]);
            sub_12F48(v41);
            v25 = (__int16 **)sub_37EDC(v39, (int)v38, (char *)aHelpMap);
            if ( v25 != v36 )
              sub_17890((int)v36, *v25, v25[1]);
            sub_12F48(v39);
            v26 = (wchar_t *)operator new(0x830u);
            if ( v26 != nullptr )
              v27 = sub_3F8FC(v26, v3[2], v3[17], 3, (wchar_t *)aF);
            else
              v27 = nullptr;
            v3[13] = (int)v27;
            if ( v27 == nullptr )
              goto LABEL_49;
            sub_43D54(v27, *(_DWORD *)(v3[1] + 16));
            v28 = sub_132FC(v44, v36);
            v29 = sub_132FC(v45, v37);
            (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v3[13] + 4))(
              v3[13],
              *v29,
              v29[1],
              v29[2],
              *v28,
              v28[1],
              v28[2]);
            sub_12EA0((int)v36);
            sub_12EA0((int)v37);
            return sub_12EA0((int)v38);
          case 4:
            sub_1309C((int)v38);
            sub_162DC(v38, (char *)aData);
            v30 = sub_37EDC(v40, (int)v38, (char *)aUiPak);
            sub_161B0((int)v37, (int)v30);
            sub_12EA0((int)v40);
            v31 = sub_37EDC(v42, (int)v38, (char *)aHelpMap);
            sub_161B0((int)v36, (int)v31);
            sub_12EA0((int)v42);
            v32 = (wchar_t *)operator new(0x830u);
            if ( v32 != nullptr )
              v33 = sub_3F8FC(v32, v3[2], v3[17], 2, word_7F128);
            else
              v33 = nullptr;
            v3[13] = (int)v33;
            if ( v33 == nullptr )
              goto LABEL_49;
            sub_43D54(v33, *(_DWORD *)(v3[1] + 16));
            v34 = sub_132FC(v47, v36);
            v35 = sub_132FC(v49, v37);
            (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v3[13] + 4))(
              v3[13],
              *v35,
              v35[1],
              v35[2],
              *v34,
              v34[1],
              v34[2]);
            sub_12EA0((int)v36);
            sub_12EA0((int)v37);
            return sub_12EA0((int)v38);
          case 5:
            *(_BYTE *)(v3[1] + 12) = 1;
            goto LABEL_49;
          default:
            goto LABEL_49;
        }
      }
      else
      {
        if ( a3 == 38 )
        {
          v10 = v3[14];
          if ( v10 != 0 )
            v3[14] = v10 - 1;
          else
            v3[14] = 5;
        }
        else if ( a3 == 40 )
        {
          v9 = v3[14];
          if ( v9 == 5 )
            v3[14] = 0;
          else
            v3[14] = v9 + 1;
        }
LABEL_49:
        sub_12EA0((int)v36);
        sub_12EA0((int)v37);
        return sub_12EA0((int)v38);
      }
    }
  }
  return result;
}


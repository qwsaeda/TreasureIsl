/* ea:   0x00063AE0 */
/* name: sub_63AE0 */
/* size: 264 */
HBITMAP __fastcall sub_63AE0(LONG a1, int a2)
{
  BITMAPINFO *v4; // r0
  BITMAPINFO *v5; // r6
  HBITMAP result; // r0
  HDC DC; // r4
  HBITMAP v8; // r5
  void *v9; // [sp+8h] [bp-28h] BYREF
  _DWORD v10[3]; // [sp+10h] [bp-20h] BYREF

  v4 = (BITMAPINFO *)malloc(0x34u);
  v5 = v4;
  if ( v4 == nullptr )
    return nullptr;
  v4->bmiHeader.biBitCount = 16;
  v4->bmiHeader.biSize = 40;
  v4->bmiHeader.biWidth = a1;
  v4->bmiHeader.biHeight = -a2;
  v4->bmiHeader.biPlanes = 1;
  v4->bmiHeader.biCompression = 3;
  v4->bmiHeader.biSizeImage = (((unsigned int)(16 * a1 + 31) >> 3) & 0x1FFFFFFC) * a2;
  v4->bmiHeader.biXPelsPerMeter = 0;
  v4->bmiHeader.biYPelsPerMeter = 0;
  v4->bmiHeader.biClrUsed = 0;
  v4->bmiHeader.biClrImportant = 0;
  v10[0] = 63488;
  v10[1] = 2016;
  v10[2] = 31;
  memcpy(v4->bmiColors, v10, 0xCu);
  DC = GetDC(nullptr);
  v8 = CreateDIBSection(DC, v5, 0, &v9, nullptr, 0);
  ReleaseDC(nullptr, DC);
  free(v5);
  if ( v9 == nullptr )
    return nullptr;
  result = v8;
  if ( v8 == nullptr )
    return nullptr;
  return result;
}


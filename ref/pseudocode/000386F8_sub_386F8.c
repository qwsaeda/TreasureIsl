/* ea:   0x000386F8 */
/* name: sub_386F8 */
/* size: 20 */
void __fastcall sub_386F8(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}


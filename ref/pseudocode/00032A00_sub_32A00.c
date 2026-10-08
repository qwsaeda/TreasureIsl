/* ea:   0x00032A00 */
/* name: sub_32A00 */
/* size: 20 */
void __fastcall sub_32A00(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

